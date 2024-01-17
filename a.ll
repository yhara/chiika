; ModuleID = 'main'
source_filename = "main"

declare ptr @sleep_sec(ptr, ptr, i64)

declare i64 @print(i64)

declare i64 @chiika_start_tokio(i64)

declare i64 @chiika_env_ref(ptr, i64)

declare i64 @chiika_env_pop(ptr, i64)

declare i64 @chiika_env_push(ptr, i64)

define i64 @chiika_main() {
start:
  %result = call i64 @foo(i64 1234)
  %result1 = call i64 @print(i64 %result)
  ret i64 0
}

define i64 @foo(i64 %0) {
start:
  %i = alloca i64, align 8
  store i64 0, ptr %i, align 4
  %result = icmp slt i64 %0, 99
  %cast = sext i1 %result to i64
  ret i64 %cast
}

define i64 @main() {
start:
  %result = call i64 @chiika_start_tokio(i64 0)
  ret i64 0
}
