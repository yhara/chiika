NAME = "a"
CARGO_TARGET = ENV["SHIIKA_CARGO_TARGET"] || "./target"
SRC = Dir["src/**/*"]
RUNTIME = Dir["chiika_runtime/**/*"]
RUNTIME_A = File.expand_path "#{CARGO_TARGET}/debug/libchiika_runtime.a"
CLANG = RUBY_PLATFORM =~ /linux/ ? "clang-16" : "clang"

file RUNTIME_A => [*RUNTIME] do
  cd "chiika_runtime" do
    sh "cargo build"
  end
end

file "#{NAME}.bc" => [*SRC] do
  sh "cargo run"
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

task :aout => "#{NAME}.out" do
  sh "./a.out"
end

task :a do
  cd "chiika-2" do
    sh "cargo run"
  end
end

