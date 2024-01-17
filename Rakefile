NAME = "a"
CARGO_TARGET = ENV["SHIIKA_CARGO_TARGET"] || "./target"
SRC_1 = Dir["chiika-1/src/**/*"]
SRC_2 = Dir["chiika-2/src/**/*"]
RUNTIME = Dir["chiika_runtime/**/*"]
RUNTIME_A = File.expand_path "#{CARGO_TARGET}/debug/libchiika_runtime.a"
CLANG = RUBY_PLATFORM =~ /linux/ ? "clang-16" : "clang"

file RUNTIME_A => [*RUNTIME] do
  cd "chiika_runtime" do
    sh "cargo fmt"
    sh "cargo build"
  end
end

file "#{NAME}.bc" => [*SRC_1, "#{NAME}.chiika1"] do
  cd "chiika-1" do
    sh "cargo fmt"
    sh "cargo run -- ../#{NAME}.chiika1"
  end
end

file "#{NAME}.out" => [RUNTIME_A, "#{NAME}.bc"] do
  sh CLANG,
    "-lm",
    "-ldl",
    "-lpthread",
    "-o", "#{NAME}.out",
    "#{NAME}.bc",
    RUNTIME_A
end

file "#{NAME}.chiika1" => [*SRC_2, "#{NAME}.chiika2"] do
  cd "chiika-2" do
    sh "cargo fmt"
    sh "cargo run -- ../#{NAME}.chiika2 > ../#{NAME}.chiika1"
  end
end

task "a" => "#{NAME}.out" do
  sh "./a.out"
end

task "2" => "#{NAME}.chiika1"
task default: "a"
