// Integration test for SigmaOS-arch-power-shell
// Verifies the crate builds and links as a dependency.

use sigmaos_arch_power_shell::PowerShell;

#[test]
fn crate_links_and_runs() {
    // Arrange
    let ps = PowerShell::new();

    // Act
    let output = ps.run_line("Get process");

    // Assert
    assert!(output.is_ok());
    assert!(output.unwrap().contains("Retrieved"));
}
