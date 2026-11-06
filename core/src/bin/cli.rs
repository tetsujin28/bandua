//! Outil en ligne de commande pour tester le moteur sans interface.
//! bandua-cli <source> <dest1> [dest2 ...] [--project X] [--day J01] [--camera A] [--card A001] [--algo xxh64|xxh3|md5|sha1]
//! bandua-cli --verify <fichier.mhl>

use bandua_core::{run, verify_mhl, Algo, Event, Job};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cancel = AtomicBool::new(false);
    if args.first().map(|s| s.as_str()) == Some("--verify") {
        let mhl = PathBuf::from(args.get(1).expect("chemin du .mhl"));
        match verify_mhl(&mhl, &cancel) {
            Ok((n, errs)) if errs.is_empty() => println!("OK : {n} fichiers re-vérifiés"),
            Ok((n, errs)) => {
                println!("{n} fichiers, {} erreur(s) :", errs.len());
                errs.iter().for_each(|e| println!("  {e}"));
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(2)
            }
        }
        return;
    }
    let mut pos = vec![];
    let mut job = Job {
        source: PathBuf::new(),
        destinations: vec![],
        project: String::new(),
        day: String::new(),
        camera: String::new(),
        card: String::new(),
        operator: String::new(),
        notes: String::new(),
        algo: Algo::Xxh64,
        lang: "fr".into(),
    };
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--project" => job.project = it.next().unwrap_or_default(),
            "--day" => job.day = it.next().unwrap_or_default(),
            "--camera" => job.camera = it.next().unwrap_or_default(),
            "--card" => job.card = it.next().unwrap_or_default(),
            "--operator" => job.operator = it.next().unwrap_or_default(),
            "--notes" => job.notes = it.next().unwrap_or_default(),
            "--lang" => job.lang = it.next().unwrap_or_else(|| "fr".into()),
            "--algo" => {
                job.algo = match it.next().as_deref() {
                    Some("xxh3") => Algo::Xxh3,
                    Some("md5") => Algo::Md5,
                    Some("sha1") => Algo::Sha1,
                    _ => Algo::Xxh64,
                }
            }
            _ => pos.push(PathBuf::from(a)),
        }
    }
    if pos.len() < 2 {
        eprintln!("usage : bandua-cli <source> <destination> [destination…] [options]");
        std::process::exit(2);
    }
    job.source = pos.remove(0);
    job.destinations = pos;
    let res = run(&job, &cancel, &mut |e| match e {
        Event::Log { level, message } => println!("[{level}] {message}"),
        Event::FileResult { result, .. } if !result.ok => println!("  ERREUR {}", result.rel),
        _ => {}
    });
    match res {
        Ok(s) => {
            for r in &s.reports {
                println!("rapport : {}", r.display());
            }
            if !s.success() {
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    }
}
