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
      sha256 "960b47cdeb5cc0204edb39c3b25e328c5333c5a02a2357bded9481e2ff59edf4"
    end
    on_intel do
      url "https://github.com/casoon/auditmysite/releases/download/v#{version}/audit-x86_64-apple-darwin.tar.gz"
      sha256 "7afb017ffa6badf7575a958636de14b50e62526d340307d90fd240ceb8f1c6f1"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/casoon/auditmysite/releases/download/v#{version}/audit-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "548ddfe2fc82bf6ba7403275bd479348c42ae326f5b5a1c9649247bd89a49f4a"
    end
    on_intel do
      url "https://github.com/casoon/auditmysite/releases/download/v#{version}/audit-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "ab1a7bac5eea358ae5204aebdf9d92c79dc9035ab2ec95f1eab52e209df715b3"
    end
  end

  def install
    bin.install "audit"
  end

  test do
    system "#{bin}/audit", "--version"
  end
end
