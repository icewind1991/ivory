extern crate bindgen;
extern crate cc;
extern crate num_cpus;

use bindgen::Builder;
use std::env;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

const PHP_VERSION: &'static str = concat!("php-", env!("CARGO_PKG_VERSION"));

/// println_stderr and run_command_or_fail are copied from rdkafka-sys
macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

fn run_command_or_fail(dir: String, cmd: &str, args: &[&str]) {
    println_stderr!(
        "Running command: \"{} {}\" in dir: {}",
        cmd,
        args.join(" "),
        dir
    );
    let ret = Command::new(cmd)
        .current_dir(dir)
        .args(args)
        .env("CC", "clang")
        .status();
    match ret.map(|status| (status.success(), status.code())) {
        Ok((true, _)) => return,
        Ok((false, Some(c))) => panic!("Command failed with error code {}", c),
        Ok((false, None)) => panic!("Command got killed"),
        Err(e) => panic!("Command failed with error: {}", e),
    }
}

fn target(path: &str) -> String {
    let osdir = env::var("PWD").unwrap();
    let pfx = match env::var("CARGO_TARGET_DIR") {
        Ok(d) => d,
        Err(_) => String::from("target"),
    };
    let profile = env::var("PROFILE").unwrap();
    format!("{}/{}/{}/native/{}", osdir, pfx, profile, path)
}

fn exists(path: &str) -> bool {
    Path::new(target(path).as_str()).exists()
}

fn compile_php(php_version: &str, link_static: bool) -> () {
    println_stderr!("Setting up PHP {}", php_version);
    run_command_or_fail("/".to_string(), "mkdir", &["-p", &target("")]);
    run_command_or_fail(
        target(""),
        "git",
        &[
            "clone",
            "https://github.com/php/php-src",
            format!("--branch={}", php_version).as_str(),
        ],
    );
    run_command_or_fail(
        target("php-src"),
        "sed",
        &[
            "-e",
            "s/void zend_signal_startup/ZEND_API void zend_signal_startup/g",
            "-ibk",
            "Zend/zend_signal.c",
            "Zend/zend_signal.h",
        ],
    );
    run_command_or_fail(target("php-src"), "./genfiles", &[]);
    run_command_or_fail(target("php-src"), "./buildconf", &["--force"]);

    let embed_type = if link_static { "static" } else { "shared" };

    #[cfg(all(target_os = "linux"))]
    let config = &[
        "--enable-debug",
        &format!("--enable-embed={}", embed_type),
        "--disable-cli",
        "--disable-cgi",
        "--enable-maintainer-zts",
        // "--without-iconv",
        "--disable-libxml",
        "--disable-dom",
        "--disable-xml",
        "--disable-simplexml",
        "--disable-xmlwriter",
        "--disable-xmlreader",
        // "--without-pear",
        // "--with-libdir=lib64",
        // "--with-pic",
    ];
    #[cfg(all(target_os = "macos"))]
    let config = &[
        "--enable-debug",
        &format!("--enable-embed={}", embed_type),
        "--disable-cli",
        "--disable-cgi",
        "--enable-maintainer-zts",
        "--without-iconv",
        "--disable-libxml",
        "--disable-dom",
        "--disable-xml",
        "--disable-simplexml",
        "--disable-xmlwriter",
        "--disable-xmlreader",
        // "--without-pear",
        // "--with-libdir=lib64",
        // "--with-pic",
    ];
    run_command_or_fail(target("php-src"), "./configure", config);
    let cpus = format!("{}", num_cpus::get());
    run_command_or_fail(target("php-src"), "make", &["-j", cpus.as_str()]);
}

fn main() {
    #[cfg(all(target_os = "linux"))]
    let default_link_static = false;
    #[cfg(all(target_os = "macos"))]
    let default_link_static = true;
    let php_version = option_env!("PHP_VERSION").unwrap_or(PHP_VERSION);

    println!("cargo:rerun-if-env-changed=PHP_VERSION");
    println!("cargo:rerun-if-env-changed=PHP_LINK_STATIC");

    let link_dynamic = env::var_os("PHP_LINK_DYNAMIC")
        .map(|_| true)
        .unwrap_or(false);
    let link_static = env::var_os("PHP_LINK_STATIC")
        .map(|_| true)
        .unwrap_or(default_link_static && !link_dynamic);

    let maybe_include_dir = Command::new("php-config")
        .args(&["--include-dir"])
        .output()
        .map(|o| String::from_utf8(o.stdout).unwrap().trim_end().to_string())
        .ok();

    let include_dir = maybe_include_dir.unwrap_or_else(|| {
        if !exists("php-src/LICENSE") {
            compile_php(php_version, link_static);
        }
        target("php-src")
    });

    let includes = ["/", "/TSRM", "/Zend", "/main"]
        .iter()
        .map(|d| format!("-I{}{}", include_dir, d))
        .collect::<Vec<String>>();

    let bindings = Builder::default()
        .rustfmt_bindings(true)
        .clang_args(includes)
        .whitelist_function("zend_error")
        .whitelist_function("php_info_print_table_start")
        .whitelist_function("php_info_print_table_row")
        .whitelist_function("php_info_print_table_end")
        .whitelist_function("php_printf")
        .whitelist_function("_zend_new_array")
        .whitelist_function("add_index_zval")
        .whitelist_function("add_assoc_zval_ex")
        .whitelist_function("zval_ptr_dtor")
        .whitelist_type("zval")
        .whitelist_type("zend_execute_data")
        .whitelist_type("zend_module_entry")
        .derive_default(false)
        .header("wrapper.h")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
