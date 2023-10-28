use sqlx::{FromRow, SqlitePool};

#[derive(FromRow, Debug)]
pub struct Tag {
    pub id: u32,
    pub value: String,
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
    pub fn tags(&self) -> &Vec<String> {
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
