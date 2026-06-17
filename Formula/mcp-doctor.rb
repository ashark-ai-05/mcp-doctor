class McpDoctor < Formula
  desc "Record, replay, and debug MCP JSON-RPC sessions from your terminal"
  homepage "https://github.com/ashark-ai-05/mcp-doctor"
  version "0.3.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/ashark-ai-05/mcp-doctor/releases/download/v0.3.0/mcp-doctor-0.3.0-macos-arm64.tar.gz"
      sha256 "5fcc2126c0386c4fe1b4cf79a42ee4ab05c7a4f9d739235a65dd82f0e5dd74fc"
    else
      url "https://github.com/ashark-ai-05/mcp-doctor/releases/download/v0.3.0/mcp-doctor-0.3.0-macos-x86_64.tar.gz"
      sha256 "e40af9ce0bf727798f32fcb411eedc89cf3d811e12438e8767d4b2ccb6e8644d"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/ashark-ai-05/mcp-doctor/releases/download/v0.3.0/mcp-doctor-0.3.0-linux-arm64-musl.tar.gz"
      sha256 "969b7df67b468e2c681fb25af560dae0abc45fd86169935faf485b321d905ce8"
    else
      url "https://github.com/ashark-ai-05/mcp-doctor/releases/download/v0.3.0/mcp-doctor-0.3.0-linux-x86_64-musl.tar.gz"
      sha256 "156d32ff9cbb5f3348bd17ce370998b40c4537d0c4850a6911fde16380049d46"
    end
  end

  def install
    bin.install "mcp-doctor"
  end

  test do
    assert_match "mcp-doctor #{version}", shell_output("#{bin}/mcp-doctor --version")
  end
end
