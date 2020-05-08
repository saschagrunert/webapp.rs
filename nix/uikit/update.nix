{ lib
, writeScript
, runtimeShell
, curl
, jq
, coreutils
, common-updater-scripts
}:

writeScript "update-uikit" ''
  #!${runtimeShell}
  PATH=${lib.makeBinPath [ curl jq coreutils common-updater-scripts ]}

  tags=$(curl -s https://api.github.com/repos/uikit/uikit/tags)
  lastest_tag=$(echo $tags | jq -r '.[] | .name' | sort --version-sort | tail -1)

  update-source-version uikit "$last_tag"
''
