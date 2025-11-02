use crate::action::AsyncAction;
use color_eyre::eyre::{bail, Result};
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

pub async fn read_dir(path: &Path) -> AsyncAction {
    if !path.exists() || path.is_dir() {
        return AsyncAction::LoadFileContents(String::new());
    }
    let res = tokio::fs::read(path).await;
    match res {
        Ok(contents) => {
            let string = String::from_utf8(contents).unwrap();
            AsyncAction::LoadFileContents(string)
        }
        Err(err) => AsyncAction::Error(format!("{:?}", err)),
    }
}

pub async fn read_dir_limited(path: &Path, lines_limit: usize) -> Result<String> {
    if lines_limit == 0 || !path.exists() || path.is_dir() {
        bail!("Limit of {lines_limit} files is invalid");
    }
    let file = match File::open(path).await {
        Ok(file) => file,
        Err(err) => bail!(err),
    };
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut count = lines_limit;
    let mut string = String::new();
    while count > 0
        && let Ok(line) = lines.next_line().await
    {
        if let Some(line) = &line {
            string += line
        }
        count -= 1;
        string += "\n";
    }
    string.pop();
    Ok(string)
}
