class MobileUse < Formula
  desc "Mobile UI automation CLI for AI agents"
  homepage "https://github.com/liuqijun/mobile-use"
  version "0.2.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/liuqijun/mobile-use/releases/download/v#{version}/mobile-use-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM64"
    else
      url "https://github.com/liuqijun/mobile-use/releases/download/v#{version}/mobile-use-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X86_64"
    end
  end

  on_linux do
    url "https://github.com/liuqijun/mobile-use/releases/download/v#{version}/mobile-use-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "PLACEHOLDER_SHA256_LINUX"
  end

  def install
    bin.install "mobile-use"
  end

  test do
    assert_match "mobile-use", shell_output("#{bin}/mobile-use --version")
  end
end
