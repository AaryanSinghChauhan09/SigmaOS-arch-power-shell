//! PowerShell-like command interpreter for SigmaOS.
//!
//! Implements cmdlet pattern, pipeline execution, and object-oriented command model.
//! Dependency-free.

use std::collections::HashMap;
use std::fmt;

/// Errors for PowerShell operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowerShellError {
    ParseError(String),
    CmdletNotFound(String),
    PipelineError(String),
    EmptyCommand,
}

impl fmt::Display for PowerShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseError(m) => write!(f, "parse error: {m}"),
            Self::CmdletNotFound(c) => write!(f, "cmdlet not found: {c}"),
            Self::PipelineError(m) => write!(f, "pipeline error: {m}"),
            Self::EmptyCommand => write!(f, "empty command"),
        }
    }
}

impl std::error::Error for PowerShellError {}

/// Cmdlet trait for command execution.
pub trait Cmdlet: std::fmt::Debug {
    fn name(&self) -> &str;
    fn execute(&self, args: &[String]) -> Result<String, PowerShellError>;
}

/// Basic cmdlet implementations.
#[derive(Debug)]
pub struct GetCmdlet;
impl Cmdlet for GetCmdlet {
    fn name(&self) -> &str { "Get" }
    fn execute(&self, args: &[String]) -> Result<String, PowerShellError> {
        if args.is_empty() {
            return Ok("Usage: Get <object>".to_string());
        }
        Ok(format!("Retrieved {}", args.join(" ")))
    }
}

#[derive(Debug)]
pub struct SetCmdlet;
impl Cmdlet for SetCmdlet {
    fn name(&self) -> &str { "Set" }
    fn execute(&self, args: &[String]) -> Result<String, PowerShellError> {
        if args.len() < 2 {
            return Ok("Usage: Set <property> <value>".to_string());
        }
        Ok(format!("Set {} = {}", args[0], args[1]))
    }
}

/// Parsed PowerShell command.
#[derive(Debug, Clone, PartialEq)]
pub struct PSCommand {
    pub cmdlet: String,
    pub args: Vec<String>,
    pub pipeline: Vec<PSCommand>,
}

/// PowerShell interpreter.
#[derive(Debug, Default)]
pub struct PowerShell {
    cmdlets: HashMap<String, Box<dyn Cmdlet>>,
}

impl PowerShell {
    pub fn new() -> Self {
        let mut ps = Self::default();
        ps.register_cmdlet(Box::new(GetCmdlet));
        ps.register_cmdlet(Box::new(SetCmdlet));
        ps
    }

    pub fn register_cmdlet(&mut self, cmdlet: Box<dyn Cmdlet>) {
        self.cmdlets.insert(cmdlet.name().to_string(), cmdlet);
    }

    pub fn parse_command(&self, input: &str) -> Result<PSCommand, PowerShellError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(PowerShellError::EmptyCommand);
        }

        // Split by pipe for pipeline
        let parts: Vec<&str> = trimmed.split('|').collect();
        let mut pipeline = Vec::new();

        for part in parts {
            let tokens: Vec<&str> = part.trim().split_whitespace().collect();
            if tokens.is_empty() {
                continue;
            }
            let cmdlet = tokens[0].to_string();
            let args = tokens[1..].iter().map(|s| s.to_string()).collect();
            pipeline.push(PSCommand { cmdlet, args, pipeline: Vec::new() });
        }

        if pipeline.is_empty() {
            return Err(PowerShellError::EmptyCommand);
        }

        let first = pipeline.remove(0);
        Ok(PSCommand { pipeline, ..first })
    }

    pub fn execute(&self, cmd: &PSCommand) -> Result<String, PowerShellError> {
        let cmdlet = self.cmdlets.get(&cmd.cmdlet)
            .ok_or_else(|| PowerShellError::CmdletNotFound(cmd.cmdlet.clone()))?;

        let mut output = cmdlet.execute(&cmd.args)?;

        // Execute pipeline
        for next in &cmd.pipeline {
            let next_cmdlet = self.cmdlets.get(&next.cmdlet)
                .ok_or_else(|| PowerShellError::CmdletNotFound(next.cmdlet.clone()))?;
            output = next_cmdlet.execute(&next.args)?;
        }

        Ok(output)
    }

    pub fn run_line(&self, line: &str) -> Result<String, PowerShellError> {
        let cmd = self.parse_command(line)?;
        self.execute(&cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_get_aaa() {
        // Arrange
        let ps = PowerShell::new();

        // Act
        let cmd = ps.parse_command("Get-Process");

        // Assert
        assert!(cmd.is_ok());
        assert_eq!(cmd.unwrap().cmdlet, "Get-Process");
    }

    #[test]
    fn test_execute_get_aaa() {
        // Arrange
        let ps = PowerShell::new();

        // Act
        let output = ps.run_line("Get process");

        // Assert
        assert!(output.is_ok());
        assert!(output.unwrap().contains("Retrieved"));
    }

    #[test]
    fn test_unknown_cmdlet_aaa() {
        // Arrange
        let ps = PowerShell::new();

        // Act
        let res = ps.run_line("Unknown-Cmdlet arg");

        // Assert
        assert!(res.is_err());
    }
}
