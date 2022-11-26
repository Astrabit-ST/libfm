# frozen_string_literal: true

require "mkmf"
require "rb_sys/mkmf"

create_rust_makefile("libfm/libfm") do |r|
  # r.force_install_rust_toolchain = "nightly"
end
