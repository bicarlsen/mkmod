#![feature(io_error_more)]
#![feature(file_create_new)]

//! Functionality for creating new modules.
pub mod result;
use std::path::{PathBuf, Path};
use crate::result::Result;
use regex::Regex;
use tempfile::NamedTempFile;
use std::io::{self, Write, BufRead};
use std::fs::{self, File};
use std::ffi::OsStr;

/// Create a new module.
///
/// # Args
/// + `name`: Name of the module.
/// + `dir`: If the module is a direcotry or a file.
/// + `add_to_super`: Automatically add the new module to it's super, if it exists. 
/// + `super_main`: Add module to main instead of lib. Only applicable if `add_to_super` is true,
/// and module is being created in the crate root.
/// + `public`: Add the module as public.
///
/// # Errors
/// + If a module of the given name already exists.
pub fn main(
    path: &Path, 
    dir: bool, 
    with_test: bool, 
    add_to_super: bool, 
    super_main: bool, 
    public: bool
) -> Result {
    if path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists, "file already exists"
        ).into());
    }

    let mod_path;
    if dir {
        mod_path = make_mod_dir(path, with_test)?;
    } else {
        mod_path = make_mod_file(path, with_test)?;
    }

    if add_to_super {
        crate::add_to_super(&mod_path, super_main, public)?;
    }

    Ok(())
}

/// Make a file module.
///
/// # Arguments
/// + `path`: Path of the module. Should not include file extensions.
/// + `with_test`: Create a test module.
///
pub fn make_mod_file(path: &Path, with_test: bool) -> Result<PathBuf> {
    // get module name
    let name = match path.file_name() {
        Some(p) => p,
        None => return Err(io::Error::new(
            io::ErrorKind::InvalidFilename, "module name could not be derived from path"
        ).into()),
    };
    let name = match name.to_str() {
        Some(p) => p,
        None => return Err(io::Error::new(
            io::ErrorKind::InvalidFilename, "module name could not convert module name to string"
        ).into()),
    };

    // create module file
    let mod_path = path.with_extension("rs");
    let mut file = File::create_new(&mod_path)?;
    if with_test {
       // create module test
        let path_str = match path.to_str() {
            Some(p) => p,
            None => return Err(io::Error::new(
                io::ErrorKind::InvalidFilename, "path could not be converted to string"
            ).into()),
        };

        let test_path = format!("{}_test.rs", path_str);
        File::create(test_path)?;
     
        // add test to module file
        let content = file_template_with_test(name);
        let content = content.into_bytes();
        file.write(&content)?;
    }

    Ok(mod_path)
}

/// Create a directory module.
///
/// # Arguments
/// + `path`: Path of the module.
/// + `with_test`: Create a test module.
pub fn make_mod_dir(path: &Path, with_test: bool) -> Result<PathBuf> {
    fs::create_dir(path)?; 

    let mod_path = path.join("mod");
    make_mod_file(&mod_path, with_test)?;

    Ok(path.to_path_buf())
}

/// Add a module to its super module.
///
/// # Argument
/// + `path`: Path of the module to add.
/// + `super_main`: Add module to main instead of lib. Only applicable if adding module to crate
/// root.
/// + `public`: Add the module as public.
pub fn add_to_super(path: &Path, super_main: bool, public: bool) -> Result {
    // get super file
    let super_file = super_path(path, super_main)?;

    // add new module to super
    let mod_name = match path.file_stem() {
        Some(p) => p,
        None => return Err(io::Error::new(
            io::ErrorKind::InvalidFilename, "could not derive module name from path"
        ).into()),
    };

    add_module_to(&mod_name, &super_file, public)
}

/// Get the super file of the given module file.
/// 
/// # Arguments
/// + `path`: Path to the module. Should be the file path for a file module,
///     or the directory for a directory module.
/// + `super_main`: Default to `main.rs`.
///
/// # Returns
/// Path to the module's super file.
fn super_path(path: &Path, super_main: bool) -> Result<PathBuf> {
    // get parent
    let abs_path = path.canonicalize()?;
    let parent = match abs_path.parent() {
        Some(p) => p,
        None => return Err(io::Error::new(
            io::ErrorKind::InvalidFilename, "parent could not be found from path"
        ).into()),
    };

    let g_parent = match parent.parent() {
        Some(p) => p,
        None => return Err(io::Error::new(
            io::ErrorKind::InvalidFilename, "grandparent could not be found from path"
        ).into()),
    };

    let cargo_file = g_parent.join("Cargo.toml");
    let parent_is_root = cargo_file.exists();

    let super_file: PathBuf;
    if parent_is_root{
        if super_main {
            super_file = parent.join("main.rs");
        } else {
            let lib_file = parent.join("lib.rs"); 
            if lib_file.exists() {
                super_file = lib_file;
            } else {
                // fall back to main.rs
                super_file = parent.join("main.rs"); 
            }
        } 
    } else {
        super_file = parent.join("mod.rs");
    }

    if !super_file.exists() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput, "parent module does not exist"
        ).into());
    }

    Ok(super_file)
}


/// Adds a submodule.
///
/// # Arguments
/// + `mod_name`: Name of the module to be added.
/// + `path`: Path of the file to which the module should be added.
/// + `public`: Insert the module as public.
fn add_module_to(mod_name: &OsStr, path: &Path, public: bool) -> Result {
    // get module name
    let mod_name = match mod_name.to_str() {
        Some(p) => p,
        None => return Err(io::Error::new(
                io::ErrorKind::InvalidFilename, "invalid module name"
        ).into()),
    };

    // get file info
    let (
        preamble_exists,
        preamble_end, 
        header_comment_exists, 
        header_comment_end
    ) = file_info(path)?;

    // calculate insert line
    let insert;
    if preamble_exists {
        // add new module to end of preamble
        if preamble_end.is_none() {
            // preamble ends file
            // append new module
            insert = None;
        } else {
            // preamble exists
            // add new module to end
            insert = Some(preamble_end.unwrap() + 1);
        }
    } else if header_comment_exists {
        // add new module after header comment
        if header_comment_end.is_none() {
            // header comment ends file
            // append new module
            insert = None;
        } else {
            insert = Some(header_comment_end.unwrap() + 1);
        }
        
    } else {
        // add new module to top of file
        insert = Some(0);
    }
    
    // insert module
    insert_mod_at_line(&mod_name, insert, path, public)
}

/// Gets info on the given file.
///
/// # Returns
/// A tuple of (`preamble_exists`, `preamble_end`, `header_comment_exists`, `header_comment_end`)
/// where
/// + `preamble_exists`: Whether `use` and `mod` statements exist in the file.
/// + `preamble_end`: An Option of None if the file ended with or before the preamble ended,
///     or Some(num) for the ending line of the preamble.
/// + `header_comment_exists`: Whether the file starts with a comment.
/// + `preamble_end`: An Option of None if the file ended with or before the header comment ended,
///     or Some(num) for the ending line of the header comment. 
fn file_info(path: &Path) -> Result<(bool, Option<usize>, bool, Option<usize>)> {
    let file = File::open(path)?;

    // find end of preamble
    let re_use = Regex::new(r"^\s*use\s+")?;
    let re_mod = Regex::new(r"^\s*(?:pub)?\s*mod")?;
    let re_comment = Regex::new(r"^\s*//")?; // @todo: Include C++-style comments

    let lines = io::BufReader::new(file).lines();
    let mut preamble_exists = false;
    let mut header_comment_exists = false;
    let mut content_start = false;
    let mut body_start = false;
    let mut preamble_end = None;
    let mut header_comment_end = None;
    for (l_num, line) in lines.enumerate() {
        if let Err(err) = line {
            return Err(err.into());
        }     

        let line = line.unwrap();
        
        if !content_start && line.trim().is_empty() {
            // ignore leading blank lines
            continue;
        }
        content_start = true;

        if !body_start {
            // check for leading comment
            let comment_line = re_comment.is_match(&line);
            if comment_line {
                if !header_comment_exists {
                    header_comment_exists = true;
                }

                continue;
            }
            else if !comment_line {
                if header_comment_exists {
                    header_comment_end = Some(l_num - 1);
                }
                body_start = true;
            }
        }

        // check for preamble lines
        let preamble_line = re_use.is_match(&line) || re_mod.is_match(&line);
        match (preamble_line, preamble_exists) {
            (true, false) => preamble_exists = true,
            (false, true) => {
                preamble_end = Some(l_num - 1);
                break;
            },
            _ => {},
        }
    }

    Ok((preamble_exists, preamble_end, header_comment_exists, header_comment_end))
}

/// Inserts the given module name as a public module in the given file.
///
/// # Arguments
/// + `mod_name`: Name of the module to insert.
/// + `insert`: Line at which to insert the module, or None to append at end.
/// + `path`: Path to the file in which to add the module.
/// + `public`: Whether to make the module public.
fn insert_mod_at_line(mod_name: &str, insert: Option<usize>, path: &Path, public: bool) -> Result {
    // format mod line
    let mod_str = match public {
        true => format!("pub mod {mod_name};"),
        false => format!("mod {mod_name};"),
    };

    // copy original file content to temp file
    // inserting new mod line
    let mut tmp = NamedTempFile::new()?;
    let file = File::open(path)?;
    let md = file.metadata()?; // used to check if file size is 0
                               // if so iteration over lines does not occur

    let lines = io::BufReader::new(file).lines();
    for (l_num, line) in lines.enumerate() {
        if let Err(err) = line {
            return Err(err.into());
        }     

        if insert == Some(l_num) {
            // add mod line
            writeln!(tmp, "{}", &mod_str)?;
        }

        // copy line
        let line = line.unwrap();
        writeln!(tmp, "{}", &line)?; 
    }

    if insert == None || md.len() == 0 {
        // append mod line
        writeln!(tmp, "{}", &mod_str)?;
    }

    // mv temp file to path
    fs::rename(tmp.path(), path)?;
    Ok(())
}

/// Template for file module contents.
///
/// # Arguments
/// + `name`: Name of the module.
fn file_template_with_test(name: &str) -> String {
    format!(r#"
#[cfg(test)]
#[path = "./{}_test.rs"]
mod {}_test;
"#, name, name)
}


#[cfg(test)]
#[path = "lib_test.rs"]
mod lib_test;
