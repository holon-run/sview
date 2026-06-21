#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 3 ]; then
  echo "usage: $0 <version-tag> <checksums-file> <output-file>" >&2
  exit 2
fi

version="$1"
checksums_file="$2"
output_file="$3"

if [[ "$version" != v*.*.* ]]; then
  echo "version tag must look like vX.Y.Z: $version" >&2
  exit 2
fi

if [ ! -f "$checksums_file" ]; then
  echo "checksums file does not exist: $checksums_file" >&2
  exit 2
fi

checksum_for() {
  local asset="$1"
  awk -v asset="$asset" '
    {
      file = $2
      sub(/^.*\//, "", file)
      if (file == asset) {
        print $1
        exit
      }
    }
  ' "$checksums_file"
}

linux_amd64_sha="$(checksum_for sview-linux-amd64.tar.gz)"
darwin_amd64_sha="$(checksum_for sview-darwin-amd64.tar.gz)"
darwin_arm64_sha="$(checksum_for sview-darwin-arm64.tar.gz)"

for required in linux_amd64_sha darwin_amd64_sha darwin_arm64_sha; do
  if [ -z "${!required}" ]; then
    echo "missing checksum: $required" >&2
    exit 1
  fi
done

version_no_v="${version#v}"
mkdir -p "$(dirname "$output_file")"

cat > "$output_file" <<EOF
class Sview < Formula
  desc "Agent-friendly structure views of source and document files"
  homepage "https://github.com/holon-run/sview"
  version "$version_no_v"
  license "Apache-2.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/holon-run/sview/releases/download/$version/sview-darwin-arm64.tar.gz"
      sha256 "$darwin_arm64_sha"
    else
      url "https://github.com/holon-run/sview/releases/download/$version/sview-darwin-amd64.tar.gz"
      sha256 "$darwin_amd64_sha"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/holon-run/sview/releases/download/$version/sview-linux-amd64.tar.gz"
      sha256 "$linux_amd64_sha"
    else
      odie "sview does not publish a Linux ARM64 binary yet"
    end
  end

  def install
    bin.install "sview"
  end

  test do
    assert_match "$version_no_v", shell_output("#{bin}/sview --version")
  end
end
EOF
