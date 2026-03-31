# Homebrew formula for auditmysite
# Install: brew install casoon/tap/auditmysite
# Or from local tap: brew tap casoon/tap && brew install auditmysite

class Auditmysite < Formula
  desc "WCAG 2.1 accessibility checker using Chrome's Accessibility Tree"
  homepage "https://github.com/casoon/auditmysite"
  version "0.4.4"
  license "LGPL-3.0-or-later"

  on_macos do
    on_arm do
      url "https://github.com/casoon/auditmysite/releases/download/v#{version}/auditmysite-macos-arm64.tar.gz"
      # sha256 will be filled after first release
    end

    on_intel do
      url "https://github.com/casoon/auditmysite/releases/download/v#{version}/auditmysite-macos-x64.tar.gz"
      # sha256 will be filled after first release
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/casoon/auditmysite/releases/download/v#{version}/auditmysite-linux-x64.tar.gz"
      # sha256 will be filled after first release
    end
  end

  def install
    bin.install "auditmysite"
  end

  test do
    assert_match "auditmysite", shell_output("#{bin}/auditmysite --version")
  end
end
