use sqlx::{FromRow, SqlitePool};

/// A tag struct for representing query results.
/// Guaranteed to be complete and represent a valid row
#[derive(FromRow, Debug)]
pub struct DatabaseTag {
    pub id: u32,
    pub value: String,
}

impl std::convert::Into<Tag> for DatabaseTag {
    fn into(self) -> Tag {
        let Self { id, value } = self;
        return Tag {
            id: Some(id),
            value,
        };
    }
}
impl std::fmt::Display for DatabaseTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            return write!(f, "id {}: {}", self.id, self.value);
    }
}

impl DatabaseTag {
    pub async fn from_id(id: u32, pool: &SqlitePool) -> anyhow::Result<Option<Self>> {
        return Ok(sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM tags
            WHERE id=?1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?);
    }

    pub async fn from_value(value: &str, pool: &SqlitePool) -> anyhow::Result<Option<Self>> {
        return Ok(sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM tags
            WHERE value=?1
            "#,
        )
        .bind(value)
        .fetch_optional(pool)
        .await?);
    }

    pub async fn from_insert(tag: Tag, pool: &SqlitePool) -> anyhow::Result<Self> {
        match Self::from_value(&tag.value, pool).await? {
            Some(dbt) => {
                println!("Tag already exists: {:?}", dbt);
                return Ok(dbt);
            }
            None => {
                sqlx::query(
                    r#"
                    INSERT INTO tags (value)
                    VALUES (?1)
                    "#,
                )
                .bind(&tag.value)
                .execute(pool)
                .await?;
                return match Self::from_value(&tag.value, pool).await? {
                    Some(dbt) => Ok(dbt),
                    None => Err(anyhow::anyhow!(
                        "Failed to insert tag with value {}",
                        &tag.value
                    )),
                };
            }
        };
    }

    pub async fn delete(self, pool: &SqlitePool) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            DELETE FROM tags
            WHERE value=?1
            "#,
        )
        .bind(&self.value)
        .execute(pool)
        .await?;
        return Ok(());
    }

    pub async fn from_tag(tag: Tag, pool: &SqlitePool) -> anyhow::Result<Self> {
        return match Self::from_value(&tag.value, pool).await? {
            Some(dbt) => Ok(dbt),
            None => {
                // Add entry to database
                sqlx::query(
                    r#"
                        INSERT INTO tags (value)
                        VALUES (?1)
                        "#,
                )
                .bind(&tag.value)
                .execute(pool)
                .await?;
                return match Self::from_value(&tag.value, pool).await? {
                    Some(dbt) => Ok(dbt),
                    None => Err(anyhow::anyhow!(
                        "Failed to add tag with value: {}",
                        &tag.value
                    )),
                };
            }
        };
    }
}

#[derive(FromRow, Debug)]
pub struct Tag {
    pub id: Option<u32>,
    pub value: String,
}

impl Tag {
    pub fn new(value: &str) -> Self {
        return Self {
            id: None,
            value: value.to_lowercase(),
        };
    }

    pub async fn from_id(id: u32, pool: &SqlitePool) -> anyhow::Result<Self> {
        return match DatabaseTag::from_id(id, pool).await? {
            Some(dbt) => Ok(dbt.into()),
            None => Err(anyhow::anyhow!("Tag does not exist with id: {}", id))?,
        };
    }

    pub async fn insert(self, pool: &SqlitePool) -> anyhow::Result<()> {
        let _db_tag = DatabaseTag::from_insert(self, pool).await?;
        return Ok(());
    }

    pub async fn delete(self, pool: &SqlitePool) -> anyhow::Result<()> {
        return match DatabaseTag::from_value(&self.value, pool).await? {
            Some(dbt) => dbt.delete(pool).await,
            None => {
                println!("Tag not in DB: {:?}", self);
                return Ok(());
            },
        };
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self.id {
            Some(id) => write!(f, "id {}: {}", id, self.value),
            None => write!(f, "id None: {}", self.value),
        };
    }
}

#[derive(Debug)]
pub struct TagList(pub Vec<DatabaseTag>);

impl std::fmt::Display for TagList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return self.0.iter().fold(Ok(()), |result, tag| {
            result.and_then(|_| writeln!(f, "{}", tag))
        });
    }
}

impl std::ops::Deref for TagList {
    type Target = Vec<DatabaseTag>;
    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

#[derive(Clone, Debug)]
pub struct TagInputList(Vec<String>);

impl Default for TagInputList {
    fn default() -> Self {
        return Self(Vec::new());
    }
}

impl std::fmt::Display for TagInputList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{:?}", self.0);
    }
}

impl std::convert::From<&str> for TagInputList {
    fn from(value: &str) -> Self {
        let tags = value
            .split(',')
            .filter(|v| *v != "")
            .map(|v| v.trim().to_lowercase().to_string())
            .collect();
        return Self(tags);
    }
}

impl TagInputList {
    pub fn as_tags(self) -> Vec<Tag> {
        return self.0.into_iter().map(|t| Tag::new(&t)).collect();
    }

    pub fn tag_values(&self) -> &Vec<String> {
        return &self.0;
    }

    pub async fn add_to_db(&self, pool: &SqlitePool) -> anyhow::Result<()> {
        for value in self.0.iter() {
            sqlx::query(
                r#"
                INSERT INTO tags (value)
                VALUES (?)
                "#,
            )
            .bind(value)
            .execute(pool)
            .await?;
        }
        return Ok(());
    }

    pub async fn delete_from_db(&self, pool: &SqlitePool) -> anyhow::Result<()> {
        for value in self.0.iter() {
            println!("Value to be deleted: {:?}", value);
            sqlx::query(
                r#"
            DELETE FROM tags
            WHERE value=?
            "#,
            )
            .bind(value)
            .execute(pool)
            .await?;
        }
        return Ok(());
    }
}
