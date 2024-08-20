#![doc = include_str!("../README.md")]
#![deny(
    missing_docs,
    rustdoc::broken_intra_doc_links,
    elided_lifetimes_in_paths
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use anyhow::Result;
use std::env::current_dir;
use std::process::Stdio;
use tokio::fs::read_dir;
use tokio::process::Command;
use tokio_stream::wrappers::ReadDirStream;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    Git::new().run().await?;
    Ok(())
}

struct Git {
    cmds: Vec<String>,
}

impl Git {
    fn new() -> Self {
        Self {
            cmds: ["git fetch -p", "git gc"]
                .into_iter()
                .map(String::from)
                .collect(),
        }
    }

    async fn run(&self) -> Result<()> {
        let entries = ReadDirStream::new(read_dir(current_dir()?).await?);
        let mut gits = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir() && e.path().join(".git").exists());
        let mut jhs = vec![];
        let cmd = self.cmds.join(" && ");
        while let Some(git) = gits.next().await {
            let cmd_c = cmd.clone();
            let jh = tokio::spawn(async move {
                let mut cmd = Command::new("sh");
                cmd.args(["-c", &cmd_c])
                    .current_dir(git.path())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null());
                match cmd.status().await {
                    Ok(s) => match s.success() {
                        true => println!("Success: {}", git.path().as_os_str().to_string_lossy()),
                        false => println!("Failed: {}", git.path().as_os_str().to_string_lossy()),
                    },
                    Err(_) => println!("Cannot Spawn child process"),
                }
            });
            jhs.push(jh);
        }
        for jh in jhs {
            jh.await?;
        }
        Ok(())
    }
}
