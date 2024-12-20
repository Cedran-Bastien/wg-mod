use super::NewArgs;
use crate::config::asconfig_json::{AsconfigcJson, CompilerOption};
use crate::config::mod_conf::ModConf;
use crate::utils::convert_to_absolute_path::convert_to_absolute_path;
use crate::utils::file_template;
use crate::utils::file_template::write_template;
use convert_case::{Case, Casing};
use serde_json::json;
use std::path::PathBuf;
use std::{fs, result};

type Result<T> = result::Result<T, file_template::Error>;

fn template_mod_conf(args: &NewArgs, parent_dir: &PathBuf) -> Result<()> {
    fs::create_dir_all(&parent_dir)
        .map_err(file_template::Error::DirectoryCreateError)?;

    let meta = ModConf {
        package_name: args.package_name.clone(),
        version: args.version.clone(),
        name: args.name.clone(),
        description: args.description.clone(),
    };
    let file_path = &parent_dir.join("mod.json");

    meta.write_json_to_file(file_path).map_err(|e| {
        file_template::Error::FileCreateError(e, file_path.clone())
    })?;

    Ok(())
}

fn template_script_entrypoint(
    args: &NewArgs, parent_dir: &PathBuf,
) -> Result<()> {
    write_template(
        &parent_dir,
        &format!("mod_{}.py", args.name.to_case(Case::Snake)),
        "def init():
    print(\"Hello world from {{name}}\")

def fini():
    print(\"Good bye world from {{name}}\")
",
        &json!({
            "name": args.name
        }),
    )?;

    Ok(())
}

fn template_git_ignore(parent_dir: &PathBuf) -> Result<()> {
    write_template(
        &parent_dir,
        ".gitignore",
        "/.idea
/.vscode
/target
.DS_Store
",
        &json!({}),
    )?;

    Ok(())
}

fn template_ui_entrypoint(args: &NewArgs, parent_dir: &PathBuf) -> Result<()> {
    let tokens = args.package_name.split(".").collect::<Vec<_>>();
    let package_name_without_suffix = tokens[..tokens.len() - 1].join(".");

    write_template(
        parent_dir,
        &format!("{}.as", args.name.to_case(Case::Pascal)),
        "package {{package_name}} {
  import net.wg.infrastructure.base.AbstractView;

  class {{class_name}} extends AbstractView {

  }
}
",
        &json!({
            "class_name": args.name.to_case(Case::Pascal),
            "package_name": package_name_without_suffix
        }),
    )
}

fn template_ui_config(args: &NewArgs, parent_dir: &PathBuf) -> Result<()> {
    fs::create_dir_all(parent_dir)
        .map_err(file_template::Error::DirectoryCreateError)?;
    let ui_config = AsconfigcJson {
        config: "flex".to_string(),
        compiler_option: CompilerOption {
            output: "".to_string(),
            source_path: vec![],
        },
        main_class: "".to_string(),
    };

    let filename = parent_dir.join("asconfigc.json");

    Ok(ui_config
        .write_json_to_file(&filename)
        .map_err(|e| file_template::Error::FileCreateError(e, filename))?)
}

fn init_git_repository(directory: &PathBuf) -> Result<()> {
    template_git_ignore(directory)?;

    git2::Repository::init(directory)?;

    Ok(())
}

pub fn create_mod_files(args: NewArgs) -> Result<()> {
    let kebab_name =
        args.name.from_case(Case::Alternating).to_case(Case::Kebab);

    let root_path = args.directory.join(&kebab_name);

    template_mod_conf(&args, &root_path)?;

    let scripts_entrypoint_path = &root_path.join("scripts");
    template_script_entrypoint(&args, &scripts_entrypoint_path)?;

    let ui_path = &root_path.join("ui");
    let mut ui_sources_path = ui_path.join("src");
    let tokens = args.package_name.split(".").collect::<Vec<_>>();
    for token in tokens[..tokens.len() - 1].iter() {
        ui_sources_path = ui_sources_path.join(token);
    }

    template_ui_entrypoint(&args, &ui_sources_path)?;
    template_ui_config(&args, &ui_path)?;

    init_git_repository(&root_path)?;

    let absolute_mod_path = convert_to_absolute_path(&root_path)?;
    println!("Success! Created {kebab_name} at {absolute_mod_path}");

    Ok(())
}

pub fn template_nvm_config(parent_dir: &PathBuf) -> Result<()> {
    write_template(
        parent_dir,
        "settings.txt",
        "root: {{nvm_dir}}\n
path: {{nvm_dir}}\\nodejs\n
arch: 64\n
proxy: none\n",
        &json!({
            "nvm_dir": parent_dir
        }),
    )
}

pub fn create_nvm_executable(parent_dir: &PathBuf, name: &str) -> Result<()> {
    write_template(
        parent_dir,
        name,
        "[ -s \"$NVM_DIR/nvm.sh\" ] && \\. \"$NVM_DIR/nvm.sh\"  # This loads nvm
[ -s \"$NVM_DIR/bash_completion\" ] && \\. \"$NVM_DIR/bash_completion\"  # This loads nvm bash_completion
nvm $@",
        &json!({}))
}
