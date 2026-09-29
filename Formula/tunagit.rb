# Homebrew formula for tunagit.
#
# Install directly from this repository:
#   brew install https://raw.githubusercontent.com/tarabakz25/tunagit/master/Formula/tunagit.rb
#
# Or copy this file to a tap repository (e.g. homebrew-tap/Formula/tunagit.rb)
# and install with:
#   brew tap tarabakz25/tap
#   brew install tunagit
#
# Release checklist (after pushing a new vX.Y.Z tag and the release workflow
# finishes): update `version`, the four `url`s, and the four `sha256` values
# below from the SHA256SUMS.txt asset of the GitHub release.
class Tunagit < Formula
  desc "Terminal interface for local Git and GitHub CLI in one place"
  homepage "https://github.com/tarabakz25/tunagit"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/tarabakz25/tunagit/releases/download/v0.1.0/tunagit-0.1.0-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_SHA256_FROM_SHA256SUMS_TXT"
    end
    on_intel do
      url "https://github.com/tarabakz25/tunagit/releases/download/v0.1.0/tunagit-0.1.0-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_SHA256_FROM_SHA256SUMS_TXT"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/tarabakz25/tunagit/releases/download/v0.1.0/tunagit-0.1.0-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_SHA256_FROM_SHA256SUMS_TXT"
    end
    on_intel do
      url "https://github.com/tarabakz25/tunagit/releases/download/v0.1.0/tunagit-0.1.0-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_SHA256_FROM_SHA256SUMS_TXT"
    end
  end

  def install
    bin.install Dir["tunagit-*/tunagit"].first
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/tunagit --version")
  end
end
