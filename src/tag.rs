use sqlx::{FromRow, SqlitePool};

#[derive(FromRow, Debug)]
pub struct Tag {
    pub id: u32,
    pub value: String,
}

impl Tag {
    pub fn from_value(value: &str) -> Self {
        return Self {
            id: 0,
            value: value.to_string(),
        };
    }

    pub async fn from_id_in_db(id: u32, pool: &SqlitePool) -> anyhow::Result<Self> {
        let tag = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM tags
            WHERE id=?1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        return Ok(tag);
    }

    pub async fn from_value_in_db(value: &str, pool: &SqlitePool) -> anyhow::Result<Option<Self>> {
        let mut tag = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM tags
            WHERE value=?1
            "#,
        )
        .bind(value)
        .fetch_all(pool)
        .await?;

        return Ok(tag.pop());
    }

    pub async fn add_to_db(self, pool: &SqlitePool) -> anyhow::Result<()> {
        let Tag { value, .. } = self;
        let value = value.to_lowercase();

        match Tag::from_value_in_db(&value, pool).await {
            Ok(doc_opt) => {
                if let None = doc_opt {
                    // Add entry to database
                    sqlx::query(
                        r#"
                        INSERT INTO tags (value)
                        VALUES (?1)
                        "#,
                    )
                    .bind(value)
                    .execute(pool)
                    .await?;
                }
            }
            Err(e) => return Err(e),
        }

        return Ok(());
    }

    pub async fn delete_from_db(self, pool: &SqlitePool) -> anyhow::Result<()> {
        let Tag { value, .. } = self;
        println!("Got value: {}", value);
        let value = value.to_lowercase();

        match Tag::from_value_in_db(&value, pool).await {
            Ok(tag_opt) => {
                if let Some(tag) = tag_opt {
                    // Add entry to database
                    sqlx::query(
                        r#"
                        DELETE FROM tags
                        WHERE value=?1
                        "#,
                    )
                    .bind(tag.value)
                    .execute(pool)
                    .await?;
                }
            }
            Err(e) => return Err(e),
        }

        return Ok(());
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
        return self.0.into_iter().map(|t| Tag::from_value(&t)).collect();
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
