    // Empty commit
    let (_stdout, stderr) = test_env.jj_cmd_ok(&repo_path, &["absorb"]);
    insta::assert_snapshot!(stderr, @"Nothing changed.");

    // Insert first and last lines
      zsuskuln 3027ca7a 2
      kkmpptxz d0f1e8dd 1
    Working copy now at: yqosqzyt 277bed24 (empty) (no description set)
    Parent commit      : zsuskuln 3027ca7a 2
      kkmpptxz d366d92c 1
    Rebased 1 descendant commits.
    Working copy now at: vruxwmqv 32eb72fe (empty) (no description set)
    Parent commit      : zsuskuln 5bf0bc06 2
      zsuskuln 6e2c4777 2
    Working copy now at: yostqsxw 4a48490c (empty) (no description set)
    Parent commit      : zsuskuln 6e2c4777 2
    @  yostqsxw 80965bcc (no description set)
    ○  zsuskuln 6e2c4777 2
    ○  kkmpptxz d366d92c 1
    │  @@ -0,0 +1,3 @@
    ○    kkmpptxz d366d92c 1
    │ ○  yqosqzyt hidden c506fbc7 (no description set)
    │ ○  yqosqzyt hidden 277bed24 (empty) (no description set)
    ○    kkmpptxz hidden d0f1e8dd 1
    │ ○  mzvwutvl hidden 8935ee61 (no description set)
    ○    zsuskuln 6e2c4777 2
    │ ○  vruxwmqv hidden 7b1da5cd (no description set)
    │ ○  vruxwmqv hidden 32eb72fe (empty) (no description set)
    ○  zsuskuln hidden 5bf0bc06 2
    ○    zsuskuln hidden 3027ca7a 2
    │ ○  mzvwutvl hidden 8935ee61 (no description set)
      qpvuntsm 7e885236 (conflict) 1
    Rebased 1 descendant commits.
    Working copy now at: mzvwutvl e9c3b95b (empty) (no description set)
    Parent commit      : kkmpptxz 7c36845c 2
      qpvuntsm 7e885236 (conflict) 1
    @  mzvwutvl e9c3b95b (empty) (no description set)
    ○  kkmpptxz 7c36845c 2
    ×  qpvuntsm 7e885236 (conflict) 1
       @@ -0,0 +1,10 @@
    Working copy now at: vruxwmqv 1b10dfa4 (empty) (no description set)
    @  vruxwmqv 1b10dfa4 (empty) (no description set)
    │ │  @@ -0,0 +1,1 @@
       @@ -0,0 +1,1 @@
       +0a
    ");
}

#[test]
fn test_absorb_discardable_merge_with_descendant() {
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_ok(test_env.env_root(), &["git", "init", "repo"]);
    let repo_path = test_env.env_root().join("repo");

    test_env.jj_cmd_ok(&repo_path, &["describe", "-m0"]);
    std::fs::write(repo_path.join("file1"), "0a\n").unwrap();

    test_env.jj_cmd_ok(&repo_path, &["new", "-m1"]);
    std::fs::write(repo_path.join("file1"), "1a\n1b\n0a\n").unwrap();

    test_env.jj_cmd_ok(&repo_path, &["new", "-m2", "description(0)"]);
    std::fs::write(repo_path.join("file1"), "0a\n2a\n2b\n").unwrap();

    let (_stdout, stderr) =
        test_env.jj_cmd_ok(&repo_path, &["new", "description(1)", "description(2)"]);
    insta::assert_snapshot!(stderr, @r"
    Working copy now at: mzvwutvl f59b2364 (empty) (no description set)
    Parent commit      : kkmpptxz 7e9df299 1
    Parent commit      : zsuskuln baf056cf 2
    Added 0 files, modified 1 files, removed 0 files
    ");

    // Modify first and last lines in the merge commit
    std::fs::write(repo_path.join("file1"), "1A\n1b\n0a\n2a\n2B\n").unwrap();
    // Add new commit on top
    test_env.jj_cmd_ok(&repo_path, &["new", "-m3"]);
    std::fs::write(repo_path.join("file2"), "3a\n").unwrap();
    // Then absorb the merge commit
    let (_stdout, stderr) = test_env.jj_cmd_ok(&repo_path, &["absorb", "--from=@-"]);
    insta::assert_snapshot!(stderr, @r"
    Absorbed changes into these revisions:
      zsuskuln 02668cf6 2
      kkmpptxz fcabe394 1
    Rebased 1 descendant commits.
    Working copy now at: royxmykx f04f1247 3
    Parent commit      : kkmpptxz fcabe394 1
    Parent commit      : zsuskuln 02668cf6 2
    ");

    insta::assert_snapshot!(get_diffs(&test_env, &repo_path, "mutable()"), @r"
    @    royxmykx f04f1247 3
    ├─╮  diff --git a/file2 b/file2
    │ │  new file mode 100644
    │ │  index 0000000000..31cd755d20
    │ │  --- /dev/null
    │ │  +++ b/file2
    │ │  @@ -0,0 +1,1 @@
    │ │  +3a
    │ ○  zsuskuln 02668cf6 2
    │ │  diff --git a/file1 b/file1
    │ │  index eb6e8821f1..4907935b9f 100644
    │ │  --- a/file1
    │ │  +++ b/file1
    │ │  @@ -1,1 +1,3 @@
    │ │   0a
    │ │  +2a
    │ │  +2B
    ○ │  kkmpptxz fcabe394 1
    ├─╯  diff --git a/file1 b/file1
    │    index eb6e8821f1..902dd8ef13 100644
    │    --- a/file1
    │    +++ b/file1
    │    @@ -1,1 +1,3 @@
    │    +1A
    │    +1b
    │     0a
    ○  qpvuntsm 3777b700 0
    │  diff --git a/file1 b/file1
    ~  new file mode 100644
       index 0000000000..eb6e8821f1
       --- /dev/null
       +++ b/file1
       @@ -0,0 +1,1 @@
    insta::assert_snapshot!(stderr, @r###"
    Working copy now at: kkmpptxz 74405a07 (conflict) (no description set)
      kkmpptxz 74405a07 (conflict) (no description set)
    "###);
       @@ -0,0 +1,1 @@
    // discarded because "absorb" isn't a command to squash commit descriptions.
    │  @@ -0,0 +1,7 @@
       @@ -0,0 +1,1 @@