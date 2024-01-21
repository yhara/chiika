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
  %result = call i64 @print(i64 1)
  ret i64 0
}

define i64 @main() {
start:
  %result = call i64 @chiika_start_tokio(i64 0)
  ret i64 0
}

define ptr @chiika_start_user(ptr %0, ptr %1) {
start:
  %result = call i64 @chiika_main()
  %result1 = call ptr %1(ptr %0, i64 %result)
  ret ptr %result1
}
