#![feature(fs_walk)]
#![feature(dir_entry_ext)]
#![feature(fs)]
#![feature(collections)]
#![feature(path_ext)]


use std::io;
use std::fs::{self, PathExt, DirEntry, walk_dir, Metadata};
use std::path::{Path, PathBuf};
use std::env;
use std::os;
use std::thread;

fn normalize(path: &Path) -> PathBuf {
  use std::path::Component::*;
  let mut ret = PathBuf::new();
  for component in path.components() {
    match component {
      CurDir => {},
      ParentDir => { ret.pop(); }
      _ => ret.push(component.as_os_str())
    }
  }
  ret
}



fn to_unc_path(path: &Path) -> String {
	let buf = path.to_str().unwrap().clone();
	let ns = format!(r"\\?\{}", buf);
	ns
}

fn remove_file(path: &Path, metadata: &Metadata) -> io::Result<()> {
    let mut perms = metadata.permissions();

    if (perms.readonly()) {
        perms.set_readonly(false);
        fs::set_permissions(path, perms);
    }
    let res = fs::remove_file(path);
    
    match res {
        Ok(()) => Ok(()),
        Err(e) => {
            println!("Delete failed {:?}", path);

            Err(io::Error::last_os_error())
        }
    }
}

fn abspath(path: &Path) -> PathBuf {
    let cwd = env::current_dir().unwrap();
    println!("CWD {:?}", cwd);
    let mut buf = PathBuf::new();
    buf.push(cwd);
    buf.push(path);
    normalize(buf.as_path())
   // buf.push(cwd);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let p = Path::new(&args[1]);
    let ap = abspath(&p);
    println!("Absopath ({:?})", ap );
    
    if !p.is_absolute() {
    	let cwd = env::current_dir();

        panic!("rraf: You must specify absolute path name!");


    }

    if !p.is_dir() {
    	panic!("rraf: You must specify existing directory name!");

    }

    let uncp = to_unc_path(p);
    let mut counter = 10;
    loop {
        let ok = nuke_tree(&uncp);
        if ok {
            break;
        }
        counter = counter-1;
        if counter == 0 {
            break;
        }
        thread::sleep_ms(2000);
    }
    
}

fn nuke_tree(root: &str) -> bool {
    let walker = walk_dir(root).unwrap();
    let mut failed_files = 0;
    for w in walker {
    	let ent = w.unwrap();
    	let md = ent.metadata().unwrap();
		let path = ent.path();
		if md.is_file() {
 			println!("F: {:?}", path );
            let r = remove_file(&path, &md);
            if r.is_err() {
                failed_files += 1;
            }

		} else if md.is_dir() {
			println!("D: {:?}", path );
		}

    }    
    if failed_files > 0 {
        println!("Failed files: {}", failed_files);
        return false;
    }
    let r = fs::remove_dir_all(root);
    if !r.is_err() {
        return true;
    }

    return false;

}


