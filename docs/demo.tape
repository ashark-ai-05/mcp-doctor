Output assets/demo.gif

Require python3
Require bash

Set Shell "bash"
Set FontSize 28
Set FontFamily "JetBrains Mono"
Set Width 1280
Set Height 760
Set Padding 18
Set Framerate 18
Set PlaybackSpeed 1.2
Set Theme "Dracula"
Set WindowBar Colorful
Set TypingSpeed 18ms

Type 'printf "MCP Doctor — trace, validate, replay MCP servers\\n\\n"' Enter
Sleep 800ms

Type './target/debug/mcp-doctor smoke stdio -- python3 fixtures/fake_mcp_server.py' Enter
Sleep 1s

Type './target/debug/mcp-doctor call stdio --tool echo --args "{\"text\":\"hello MCP\"}" -- python3 fixtures/fake_mcp_server.py' Enter
Sleep 1.8s

Type 'TRACE=$(ls -td .mcpdoctor/sessions/*/trace.jsonl | head -1)' Enter
Sleep 400ms

Type './target/debug/mcp-doctor validate "$TRACE"' Enter
Sleep 900ms

Type './target/debug/mcp-doctor tui "$TRACE"' Enter
Sleep 2s

Type './target/debug/mcp-doctor export-repro "$TRACE" --output /tmp/mcp-doctor-replay.sh' Enter
Sleep 1s

Type './target/debug/mcp-doctor replay "$TRACE" -- python3 fixtures/fake_mcp_server.py' Enter
Sleep 2.5s
