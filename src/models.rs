use serenity::{
    http::CacheHttp,
    model::{
        channel::Channel,
        guild::Guild,
        id::{GuildId, UserId},
        user::User,
    },
};

use sqlx::MySqlPool;

use chrono::NaiveDateTime;
use chrono_tz::Tz;

use log::error;

use crate::consts::{DEFAULT_PREFIX, LOCAL_LANGUAGE, LOCAL_TIMEZONE};

#[cfg(feature = "prefix-cache")]
use crate::PrefixCache;
use crate::SQLPool;

use serenity::prelude::Context;

pub struct GuildData {
    pub id: u32,
    pub name: Option<String>,
    pub prefix: String,
}

impl GuildData {
    #[cfg(feature = "prefix-cache")]
    pub async fn prefix_from_id<T: Into<GuildId>>(
        guild_id_opt: Option<T>,
        ctx: &Context,
    ) -> String {
        let pool = ctx.data.read().await.get::<SQLPool>().cloned().unwrap();
        let prefix_cache = ctx.data.read().await.get::<PrefixCache>().cloned().unwrap();

        if let Some(guild_id) = guild_id_opt {
            let guild_id = guild_id.into();

            if let Some(prefix) = prefix_cache.get(&guild_id) {
                prefix.to_string()
            } else {
                let row = sqlx::query!(
                    "
SELECT prefix FROM guilds WHERE guild = ?
                ",
                    guild_id.as_u64().to_owned()
                )
                .fetch_one(&pool)
                .await;

                let prefix = row.map_or_else(|_| DEFAULT_PREFIX.clone(), |r| r.prefix);

                prefix_cache.insert(guild_id, prefix.clone());

                prefix
            }
        } else {
            DEFAULT_PREFIX.clone()
        }
    }

    #[cfg(not(feature = "prefix-cache"))]
    pub async fn prefix_from_id<T: Into<GuildId>>(
        guild_id_opt: Option<T>,
        ctx: &Context,
    ) -> String {
        let pool = ctx.data.read().await.get::<SQLPool>().cloned().unwrap();

        if let Some(guild_id) = guild_id_opt {
            let guild_id = guild_id.into().as_u64().to_owned();

            let row = sqlx::query!(
                "
SELECT prefix FROM guilds WHERE guild = ?
                ",
                guild_id
            )
            .fetch_one(&pool)
            .await;

            row.map_or_else(|_| DEFAULT_PREFIX.clone(), |r| r.prefix)
        } else {
            DEFAULT_PREFIX.clone()
        }
    }

    pub async fn from_guild(guild: Guild, pool: &MySqlPool) -> Result<Self, sqlx::Error> {
        let guild_id = guild.id.as_u64().to_owned();

        match sqlx::query_as!(
            Self,
            "
SELECT id, name, prefix FROM guilds WHERE guild = ?
            ",
            guild_id
        )
        .fetch_one(pool)
        .await
        {
            Ok(mut g) => {
                g.name = Some(guild.name);

                Ok(g)
            }

            Err(sqlx::Error::RowNotFound) => {
                sqlx::query!(
                    "
INSERT INTO guilds (guild, name, prefix) VALUES (?, ?, ?)
                    ",
                    guild_id,
                    guild.name,
                    *DEFAULT_PREFIX
                )
                .execute(&pool.clone())
                .await?;

                Ok(sqlx::query_as!(
                    Self,
                    "
SELECT id, name, prefix FROM guilds WHERE guild = ?
                    ",
                    guild_id
                )
                .fetch_one(pool)
                .await?)
            }

            Err(e) => {
                error!("Unexpected error in guild query: {:?}", e);

                Err(e)
            }
        }
    }

    pub async fn commit_changes(&self, pool: &MySqlPool) {
        sqlx::query!(
            "
UPDATE guilds SET name = ?, prefix = ? WHERE id = ?
            ",
            self.name,
            self.prefix,
            self.id
        )
        .execute(pool)
        .await
        .unwrap();
    }
}

pub struct ChannelData {
    pub id: u32,
    pub name: Option<String>,
    pub nudge: i16,
    pub blacklisted: bool,
    pub webhook_id: Option<u64>,
    pub webhook_token: Option<String>,
    pub paused: bool,
    pub paused_until: Option<NaiveDateTime>,
}

impl ChannelData {
    pub async fn from_channel(
        channel: Channel,
        pool: &MySqlPool,
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let channel_id = channel.id().as_u64().to_owned();

        if let Ok(c) = sqlx::query_as_unchecked!(Self,
            "
SELECT id, name, nudge, blacklisted, webhook_id, webhook_token, paused, paused_until FROM channels WHERE channel = ?
            ", channel_id)
            .fetch_one(pool)
            .await {

            Ok(c)
        }
        else {
            let props = channel.guild().map(|g| (g.guild_id.as_u64().to_owned(), g.name));

            let (guild_id, channel_name) = if let Some((a, b)) = props {
                (Some(a), Some(b))
            } else {
                (None, None)
            };

            sqlx::query!(
                "
INSERT IGNORE INTO channels (channel, name, guild_id) VALUES (?, ?, (SELECT id FROM guilds WHERE guild = ?))
                ", channel_id, channel_name, guild_id)
                .execute(&pool.clone())
                .await?;

            Ok(sqlx::query_as_unchecked!(Self,
                "
SELECT id, name, nudge, blacklisted, webhook_id, webhook_token, paused, paused_until FROM channels WHERE channel = ?
                ", channel_id)
                .fetch_one(pool)
                .await?)
        }
    }

    pub async fn commit_changes(&self, pool: &MySqlPool) {
        sqlx::query!(
            "
UPDATE channels SET name = ?, nudge = ?, blacklisted = ?, webhook_id = ?, webhook_token = ?, paused = ?, paused_until = ? WHERE id = ?
            ", self.name, self.nudge, self.blacklisted, self.webhook_id, self.webhook_token, self.paused, self.paused_until, self.id)
            .execute(pool)
            .await.unwrap();
    }
}

pub struct UserData {
    pub id: u32,
    pub user: u64,
    pub name: String,
    pub dm_channel: u32,
    pub language: String,
    pub timezone: String,
    pub meridian_time: bool,
}

pub struct MeridianType(bool);

impl MeridianType {
    pub fn fmt_str(&self) -> &str {
        if self.0 {
            "%Y-%m-%d %I:%M:%S %p"
        } else {
            "%Y-%m-%d %H:%M:%S"
        }
    }

    pub fn fmt_str_short(&self) -> &str {
        if self.0 {
            "%I:%M %p"
        } else {
            "%H:%M"
        }
    }
}

impl UserData {
    pub async fn language_of<U>(user: U, pool: &MySqlPool) -> String
    where
        U: Into<UserId>,
    {
        let user_id = user.into().as_u64().to_owned();

        match sqlx::query!(
            "
SELECT language FROM users WHERE user = ?
            ",
            user_id
        )
        .fetch_one(pool)
        .await
        {
            Ok(r) => r.language,

            Err(_) => LOCAL_LANGUAGE.clone(),
        }
    }

    pub async fn timezone_of<U>(user: U, pool: &MySqlPool) -> Tz
    where
        U: Into<UserId>,
    {
        let user_id = user.into().as_u64().to_owned();

        match sqlx::query!(
            "
SELECT timezone FROM users WHERE user = ?
            ",
            user_id
        )
        .fetch_one(pool)
        .await
        {
            Ok(r) => r.timezone,

            Err(_) => LOCAL_TIMEZONE.clone(),
        }
        .parse()
        .unwrap()
    }

    pub async fn meridian_of<U>(user: U, pool: &MySqlPool) -> MeridianType
    where
        U: Into<UserId>,
    {
        let user_id = user.into().as_u64().to_owned();

        match sqlx::query!(
            "
SELECT meridian_time FROM users WHERE user = ?
            ",
            user_id
        )
        .fetch_one(pool)
        .await
        {
            Ok(r) => MeridianType(r.meridian_time != 0),

            Err(_) => MeridianType(false),
        }
    }

    pub async fn from_user(
        user: &User,
        ctx: impl CacheHttp,
        pool: &MySqlPool,
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let user_id = user.id.as_u64().to_owned();

        match sqlx::query_as_unchecked!(
            Self,
            "
SELECT id, user, name, dm_channel, IF(language IS NULL, ?, language) AS language, IF(timezone IS NULL, ?, timezone) AS timezone, meridian_time FROM users WHERE user = ?
            ",
            *LOCAL_LANGUAGE, *LOCAL_TIMEZONE, user_id
        )
        .fetch_one(pool)
        .await
        {
            Ok(c) => Ok(c),

            Err(sqlx::Error::RowNotFound) => {
                let dm_channel = user.create_dm_channel(ctx).await?;
                let dm_id = dm_channel.id.as_u64().to_owned();

                let pool_c = pool.clone();

                sqlx::query!(
                    "
INSERT IGNORE INTO channels (channel) VALUES (?)
                    ",
                    dm_id
                )
                .execute(&pool_c)
                .await?;

                sqlx::query!(
                    "
INSERT INTO users (user, name, dm_channel, language, timezone) VALUES (?, ?, (SELECT id FROM channels WHERE channel = ?), ?, ?)
                    ", user_id, user.name, dm_id, *LOCAL_LANGUAGE, *LOCAL_TIMEZONE)
                    .execute(&pool_c)
                    .await?;

                Ok(sqlx::query_as_unchecked!(
                    Self,
                    "
SELECT id, user, name, dm_channel, language, timezone, meridian_time FROM users WHERE user = ?
                    ",
                    user_id
                )
                .fetch_one(pool)
                .await?)
            }

            Err(e) => {
                error!("Error querying for user: {:?}", e);

                Err(Box::new(e))
            },
        }
    }

    pub async fn commit_changes(&self, pool: &MySqlPool) {
        sqlx::query!(
            "
UPDATE users SET name = ?, language = ?, timezone = ?, meridian_time = ? WHERE id = ?
            ",
            self.name,
            self.language,
            self.timezone,
            self.meridian_time,
            self.id
        )
        .execute(pool)
        .await
        .unwrap();
    }

    pub fn timezone(&self) -> Tz {
        self.timezone.parse().unwrap()
    }

    pub fn meridian(&self) -> MeridianType {
        MeridianType(self.meridian_time)
    }
}

pub struct Timer {
    pub name: String,
    pub start_time: NaiveDateTime,
    pub owner: u64,
}

impl Timer {
    pub async fn from_owner(owner: u64, pool: &MySqlPool) -> Vec<Self> {
        sqlx::query_as_unchecked!(
            Timer,
            "
SELECT name, start_time, owner FROM timers WHERE owner = ?
            ",
            owner
        )
        .fetch_all(pool)
        .await
        .unwrap()
    }

    pub async fn count_from_owner(owner: u64, pool: &MySqlPool) -> u32 {
        sqlx::query!(
            "
SELECT COUNT(1) as count FROM timers WHERE owner = ?
            ",
            owner
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .count as u32
    }

    pub async fn create(name: &str, owner: u64, pool: &MySqlPool) {
        sqlx::query!(
            "
INSERT INTO timers (name, owner) VALUES (?, ?)
            ",
            name,
            owner
        )
        .execute(pool)
        .await
        .unwrap();
    }
}
