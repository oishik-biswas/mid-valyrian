#[cfg(test)]
mod tests {
    use assert_cmd::Command;

    #[test]
    fn test_hello() {
        let mut cmd = Command::cargo_bin("mid_valyrian").unwrap();
        cmd.arg("examples/hello.mv");
        cmd.assert().success()
           .stdout(predicates::str::contains("Valar morghulis!"));
    }
}
