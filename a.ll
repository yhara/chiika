; ModuleID = 'main'
source_filename = "main"

declare ptr @sleep_sec(ptr, ptr, i64)

declare i64 @print(i64)

declare i64 @chiika_start_tokio(i64)

declare i64 @chiika_env_ref(ptr, i64)

declare i64 @chiika_env_pop(ptr, i64)

declare i64 @chiika_env_push(ptr, i64)

define ptr @chiika_main_1(ptr %0, i64 %1) {
start:
  %result = call i64 @print(i64 %1)
  %result1 = call i64 @chiika_env_pop(ptr %0, i64 1)
  %f = inttoptr i64 %result1 to ptr
  %result2 = call ptr %f(ptr %0, i64 0)
  ret ptr %result2
}

define ptr @chiika_main(ptr %0, ptr %1) {
start:
  %result = call i64 @chiika_env_push(ptr %0, i64 ptrtoint (ptr %1 to i64))
  %a = alloca i64, align 8
  store i64 579, ptr %a, align 4
  %n = load i64, ptr %a, align 4
  %result1 = call ptr @foo(ptr %0, ptr @chiika_main_1, i64 %n)
  ret ptr %result1
}

define ptr @foo_1(ptr %0, i64 %1) {
start:
  %result = call i64 @print(i64 %1)
  %result1 = call i64 @chiika_env_ref(ptr %0, i64 0)
  %result2 = call i64 @print(i64 %result1)
  %result3 = call i64 @chiika_env_pop(ptr %0, i64 2)
  %f = inttoptr i64 %result3 to ptr
  %result4 = call ptr %f(ptr %0, i64 300)
  ret ptr %result4
}

define ptr @foo(ptr %0, ptr %1, i64 %2) {
start:
  %result = call i64 @chiika_env_push(ptr %0, i64 ptrtoint (ptr %1 to i64))
  %result1 = call i64 @chiika_env_push(ptr %0, i64 %2)
  %result2 = call i64 @print(i64 100)
  %result3 = call ptr @sleep_sec(ptr %0, ptr @foo_1, i64 1)
  ret ptr %result3
}

define i64 @main() {
start:
  %result = call i64 @chiika_start_tokio(i64 0)
  ret i64 0
}
