# typed: false
# frozen_string_literal: true

# Homebrew formula for audit CLI
# This file should be copied to your homebrew-tap repository
class Audit < Formula
  desc "Resource-efficient WCAG 2.1 Accessibility Checker"
  homepage "https://github.com/casoon/auditmysite"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/casoon/auditmysite/releases/download/v#{version}/audit-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_ARM"
    end
    on_intel do
      url "https://github.com/casoon/auditmysite/releases/download/v#{version}/audit-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_INTEL"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/casoon/auditmysite/releases/download/v#{version}/audit-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_ARM"
    end
    on_intel do
      url "https://github.com/casoon/auditmysite/releases/download/v#{version}/audit-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_INTEL"
    end
  end

  def install
    bin.install "audit"
  end

  test do
    system "#{bin}/audit", "--version"
  end
end
