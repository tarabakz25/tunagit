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
      sha256 "cd171667b4a642b83dcd3c27911a63a938d83cd5a4f457571e3aa704290a0840"
    end
    on_intel do
      url "https://github.com/tarabakz25/tunagit/releases/download/v0.1.0/tunagit-0.1.0-x86_64-apple-darwin.tar.gz"
      sha256 "3830efaed9c25302a14405783304000e796f8729bb7fac4b97346e0e73bd16c1"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/tarabakz25/tunagit/releases/download/v0.1.0/tunagit-0.1.0-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "c65cec1b60b1151f0ade1f81269c05bbac687560176f7ee865ba5cb8c11edcf3"
    end
    on_intel do
      url "https://github.com/tarabakz25/tunagit/releases/download/v0.1.0/tunagit-0.1.0-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "4ce28f49f3e84fea75e66319949878d714bb7c9424e3ecc9c5235dfaee2a1518"
    end
  end

  def install
    bin.install Dir["tunagit-*/tunagit"].first
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/tunagit --version")
  end
end
