use std::{
    fs,
    path::{Path, PathBuf},
    process::Stdio,
};

use rustyline::Editor;

use crate::{
    cli::CommandContext,
    command::{get_command_output, run_command},
    config::{DOCKERCOMPOSE_DEFAULT_PATH, LENRA_CACHE_DIRECTORY},
    devtool::stop_app_env,
    docker_compose::{
        self, compose_build, compose_down, compose_up, list_running_services, Service,
    },
    errors::{Error, Result},
    git,
    template::{self, TemplateData},
};

#[cfg(test)]
use mocktopus::macros::mockable;

#[cfg_attr(test, mockable)]
pub async fn create_new_project(template: &str, path: &PathBuf) -> Result<()> {
    log::info!("Creating a new project");
    // check that the path does not exists or is empty
    if path.exists() && path.read_dir().map_err(Error::from)?.next().is_some() {
        return Err(Error::ProjectPathNotEmpty);
    }

    template::clone_template(template, path).await?;

    // create `.template` file to save template repo url and commit
    let git_dir = path.join(".git");
    let commit = git::get_current_commit(Some(git_dir.clone())).await?;
    TemplateData {
        template: template.to_string(),
        commit: Some(commit.clone()),
    }
    .save_to(&path.join(template::TEMPLATE_DATA_FILE))
    .await
    .map_err(Error::from)?;

    create_cache_directories(path, &git_dir)?;

    log::info!("Project created");
    Ok(())
}

#[cfg_attr(test, mockable)]
fn create_cache_directories(path: &PathBuf, git_dir: &PathBuf) -> Result<()> {
    log::debug!("create cache directories");
    // create the `.lenra` cache directory
    let cache_dir = path.join(LENRA_CACHE_DIRECTORY);
    fs::create_dir_all(cache_dir.clone()).unwrap();
    // move the template `.git` directory
    fs::rename(git_dir, cache_dir.join(template::TEMPLATE_GIT_DIR))?;

    log::info!("Project created");
    Ok(())
}

pub async fn generate_app_env(context: &mut CommandContext, production: bool) -> Result<()> {
    log::info!("Generating the app environment");
    let conf = context
        .config
        .clone()
        .ok_or(Error::Custom("The config is missing".into()))?;
    // TODO: check the components API version

    conf.generate_files(context, !production).await?;
    Ok(())
}

pub async fn build_app(context: &mut CommandContext) -> Result<()> {
    log::info!("Build the Docker image");
    compose_build(context).await?;
    log::info!("Image built");
    Ok(())
}

pub async fn start_env(context: &mut CommandContext) -> Result<()> {
    let dockercompose_path: PathBuf =
        context.resolve_path(&DOCKERCOMPOSE_DEFAULT_PATH.iter().collect());
    if !dockercompose_path.exists() {
        return Err(Error::NeverBuiltApp);
    }

    log::info!("Start the containers");
    compose_up(context).await?;
    let running_services: Vec<Service> = list_running_services(context).await?;
    if running_services.len() < 4 {
        return Err(Error::NotStartedServices);
    }
    Ok(())
}

pub async fn stop_env(context: &mut CommandContext) -> Result<()> {
    log::info!("Stop the containers");
    compose_down(context).await?;
    Ok(())
}

pub async fn clear_cache(context: &mut CommandContext) -> Result<()> {
    log::info!("Clearing cache");
    stop_app_env(context).await?;
    Ok(())
}

pub fn display_app_access_url() {
    println!(
        "\nApplication available at http://localhost:{}\n",
        docker_compose::DEVTOOL_WEB_PORT
    );
}

pub async fn update_env_images(
    context: &mut CommandContext,
    services: &Vec<Service>,
) -> Result<()> {
    log::info!("Update the environment images");
    docker_compose::compose_pull(context, services).await?;
    Ok(())
}

pub async fn upgrade_app() -> Result<()> {
    log::info!("Upgrading the application");
    // get template data
    let template_data = template::get_template_data().await?;
    let git_dir = Path::new(LENRA_CACHE_DIRECTORY).join(template::TEMPLATE_GIT_DIR);

    if git_dir.is_dir() {
        // update the template repo
        git::pull(Some(git_dir.clone())).await?;
    } else {
        let template_tmp = Path::new(LENRA_CACHE_DIRECTORY).join(template::TEMPLATE_TEMP_DIR);
        // clone template project
        template::clone_template(template_data.template.as_str(), &template_tmp).await?;
        fs::rename(template_tmp.join(".git"), git_dir.clone())?;
        fs::remove_dir_all(template_tmp)?;
    }

    let current_commit = git::get_current_commit(Some(git_dir.clone())).await?;
    if let Some(commit) = template_data.commit {
        if commit == current_commit {
            println!("This application is already up to date");
            return Ok(());
        }

        // get diff between previous commit and current commit
        let patch_file = Path::new(LENRA_CACHE_DIRECTORY)
            .join(format!("patch.{}-{}.diff", commit, current_commit));
        log::debug!(
            "create patch between {} and {}: {:?}",
            commit,
            current_commit,
            patch_file
        );
        let mut cmd = git::create_git_command();
        cmd.arg("--git-dir")
            .arg(git_dir.as_os_str())
            .arg("diff")
            .arg(commit)
            .arg(current_commit.clone());
        let mut patch = get_command_output(cmd).await?;
        patch.push('\n');
        fs::write(patch_file.clone(), patch)?;

        // apply a patch
        log::debug!("apply patch on project");
        let mut cmd = git::create_git_command();
        cmd.arg("apply")
            .arg(patch_file.clone())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        let patch_file_str = patch_file.to_string_lossy();
        while !cmd.spawn()?.wait_with_output().await?.status.success() {
            println!("An error occured applying the patch {patch_file_str}");
            let mut rl = Editor::<()>::new()?;
            rl.readline("Fix it and press enter to retry")?;
        }
        fs::remove_file(patch_file)?;
    } else {
        // ask for user confirmation
        if !confirm_checkout()? {
            println!("Upgrade canceled");
            return Ok(());
        }

        // checkout the template in the current dir
        log::debug!("checkout the template");
        let mut cmd = git::create_git_command();
        cmd.arg("--git-dir")
            .arg(git_dir.as_os_str())
            .arg("checkout")
            .arg("HEAD")
            .arg("--")
            .arg(".");

        run_command(cmd).await?;
    }
    // save template data
    TemplateData {
        template: template_data.template,
        commit: Some(current_commit),
    }
    .save()
    .await
}

fn confirm_checkout() -> Result<bool> {
    let mut rl = Editor::<()>::new()?;
    println!("There is no template last commit in this project, the template files will checked out to your app.\nMake sure your project is saved (for example with git).");
    loop {
        let res = rl
            .readline("Checkout the template ? [y/N] ")?
            .trim()
            .to_lowercase();
        if res == "y" || res == "yes" {
            return Ok(true);
        } else if res.is_empty() || res == "n" || res == "no" {
            return Ok(false);
        }
    }
}
