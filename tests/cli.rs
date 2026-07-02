mod cli {

    use assert_cmd::cargo::cargo_bin_cmd;
    use predicates::str::contains;

    #[tokio::test]
    async fn test_cli_version_subcommand_returns_version() {
        let mut cmd = cargo_bin_cmd!();
        cmd.arg("version")
            .assert()
            .success()
            .code(0)
            .stdout(contains(format!(
                "maedic version: {}",
                env!("CARGO_PKG_VERSION")
            )));
    }
}
