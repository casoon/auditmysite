# typed: false
# frozen_string_literal: true

# Homebrew formula for auditmysite CLI
# This file should be copied to your homebrew-tap repository
class Auditmysite < Formula
  desc "Lightning-fast WCAG accessibility auditing written in Rust"
  homepage "https://github.com/casoon/auditmysite"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/casoon/auditmysite/releases/download/v#{version}/auditmysite-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM64"
    end
    on_intel do
      url "https://github.com/casoon/auditmysite/releases/download/v#{version}/auditmysite-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X86_64"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/casoon/auditmysite/releases/download/v#{version}/auditmysite-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_ARM64"
    end
    on_intel do
      url "https://github.com/casoon/auditmysite/releases/download/v#{version}/auditmysite-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X86_64"
    end
  end

  def install
    bin.install "auditmysite"
  end

  test do
    system "#{bin}/auditmysite", "--version"
  end
end
