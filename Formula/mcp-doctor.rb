class McpDoctor < Formula
  desc "Record, replay, and debug MCP JSON-RPC sessions from your terminal"
  homepage "https://github.com/ashark-ai-05/mcp-doctor"
  version "0.4.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/ashark-ai-05/mcp-doctor/releases/download/v0.4.0/mcp-doctor-0.4.0-macos-arm64.tar.gz"
      sha256 "ce178de98e5198ddc730f12dc086e74f7056f70c5222afbb36278cb1f649e25b"
    else
      url "https://github.com/ashark-ai-05/mcp-doctor/releases/download/v0.4.0/mcp-doctor-0.4.0-macos-x86_64.tar.gz"
      sha256 "9de944c9152a3af56d4b205ddb29b04ab321ec5f20e58d86071180b88ada9c2e"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/ashark-ai-05/mcp-doctor/releases/download/v0.4.0/mcp-doctor-0.4.0-linux-arm64-musl.tar.gz"
      sha256 "ab9ae848f2352df51012d2c050327bd1cabbe2eb6ba2151fdf9329c5d658b036"
    else
      url "https://github.com/ashark-ai-05/mcp-doctor/releases/download/v0.4.0/mcp-doctor-0.4.0-linux-x86_64-musl.tar.gz"
      sha256 "348602dee2c39a7f824f2478b52d77c94b9fe1f72867b7a1876641ff6fca38ef"
    end
  end

  def install
    bin.install "mcp-doctor"
  end

  test do
    assert_match "mcp-doctor #{version}", shell_output("#{bin}/mcp-doctor --version")
  end
end
