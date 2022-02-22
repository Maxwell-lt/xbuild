use crate::devices::{Backend, Device};
use crate::{Arch, BuildEnv, Platform};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug)]
pub struct IMobileDevice {
    idevice_id: PathBuf,
    ideviceinfo: PathBuf,
    ideviceinstaller: PathBuf,
    idevicesyslog: PathBuf,
}

impl IMobileDevice {
    pub fn which() -> Result<Self> {
        Ok(Self {
            idevice_id: which::which("idevice_id")?,
            ideviceinfo: which::which("ideviceinfo")?,
            ideviceinstaller: which::which("ideviceinstaller")?,
            idevicesyslog: which::which("idevicesyslog")?,
        })
    }

    fn getkey(&self, device: &str, key: &str) -> Result<String> {
        let output = Command::new(&self.ideviceinfo)
            .arg("--udid")
            .arg(device)
            .arg("--key")
            .arg(key)
            .output()?;
        if !output.status.success() {
            anyhow::bail!("failed to run ideviceinfo");
        }
        Ok(std::str::from_utf8(&output.stdout)?.trim().to_string())
    }

    fn install(&self, device: &str, path: &Path) -> Result<()> {
        let status = Command::new(&self.ideviceinstaller)
            .arg("--udid")
            .arg(device)
            .arg("--install")
            .arg(path)
            .status()?;
        if !status.success() {
            anyhow::bail!("failed to run ideviceinstaller");
        }
        Ok(())
    }

    pub fn run(
        &self,
        device: &str,
        path: &Path,
        _env: &BuildEnv,
        _flutter_attach: bool,
    ) -> Result<()> {
        // TODO: stop, start, log, attach
        self.install(device, path)?;
        Ok(())
    }

    pub fn devices(&self, devices: &mut Vec<Device>) -> Result<()> {
        let output = Command::new(&self.idevice_id)
            .arg("-l")
            .arg("-d")
            .output()?;
        if !output.status.success() {
            anyhow::bail!("failed to run idevice_id");
        }
        let lines = std::str::from_utf8(&output.stdout)?.lines();
        for uuid in lines {
            devices.push(Device {
                backend: Backend::Imd(self.clone()),
                id: uuid.trim().to_string(),
            });
        }
        Ok(())
    }

    pub fn name(&self, device: &str) -> Result<String> {
        self.getkey(device, "DeviceName")
    }

    pub fn platform(&self, _device: &str) -> Result<Platform> {
        Ok(Platform::Ios)
    }

    pub fn arch(&self, device: &str) -> Result<Arch> {
        match self.getkey(device, "CPUArchitecture")?.as_str() {
            "arm64" => Ok(Arch::Arm64),
            arch => anyhow::bail!("unsupported arch {}", arch),
        }
    }

    pub fn details(&self, device: &str) -> Result<String> {
        let name = self.getkey(device, "ProductName")?;
        let version = self.getkey(device, "ProductVersion")?;
        Ok(format!("{} {}", name, version))
    }
}