; ModuleID = 'main'
source_filename = "main"

declare ptr @sleep_sec(ptr, ptr, i64)

declare i64 @print(i64)

declare i64 @chiika_start_tokio(i64)

declare ptr @chiika_env_pop(ptr)

declare i64 @chiika_env_push(ptr, ptr)

define ptr @chiika_main_1(ptr %0, i64 %1) {
start:
  %result = call i64 @print(i64 %1)
  %result1 = call ptr @chiika_env_pop(ptr %0)
  %result2 = call ptr %result1(ptr %0, i64 0)
  ret ptr %result2
}

define ptr @chiika_main(ptr %0, ptr %1) {
start:
  %result = call i64 @chiika_env_push(ptr %0, ptr %1)
  %result1 = call ptr @foo(ptr %0, ptr @chiika_main_1)
  ret ptr %result1
}

define ptr @foo_1(ptr %0, i64 %1) {
start:
  %result = call i64 @print(i64 %1)
  %result1 = call i64 @print(i64 200)
  %result2 = call ptr @chiika_env_pop(ptr %0)
  %result3 = call ptr %result2(ptr %0, i64 300)
  ret ptr %result3
}

define ptr @foo(ptr %0, ptr %1) {
start:
  %result = call i64 @chiika_env_push(ptr %0, ptr %1)
  %result1 = call i64 @print(i64 100)
  %result2 = call ptr @sleep_sec(ptr %0, ptr @foo_1, i64 1)
  ret ptr %result2
}

define i64 @main() {
start:
  %result = call i64 @chiika_start_tokio(i64 0)
  ret i64 0
}
