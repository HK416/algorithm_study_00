# Algorithm Study - Lamport's Bakery Algorithm
> 본 내용은 알고리즘을 공부하면서 정리한 글 입니다.


</br>

## 알고리즘 개요
둘 이상의 스레드가 동일한 메모리 주소에 접근할 경우 데이터 손상이 발생할 수 있다. </br>
'Lamport's Bakery Algorithm'은 데이터 손상의 위험을 제거하기 위해 스레드가 'critical section'에 동시에 접근하는 것을 방지하도록 하는 상호 배제 알고리즘 중 하나이다.
</br>
</br>

## 알고리즘 구현
- 비유 </br>
[1] 은행 입구에 번호표 기계가 있다. 고객은 입구에서 번호표를 뽑아 대기한다. </br>
[2] 번호표에 적힌 번호를 부르면 고객은 창구로 가서 업무를 처리한다. </br>
[3] 고객이 다시 업무를 처리하려면 번호표 기계에서 다시 번호를 뽑아야 한다.

</br>

### 의사코드
```
// 전역 변수의 선언 및 초기화
Entering: array [1..NUM_THREADS] of bool = { false };
Number: array [1..NUM_THREADS] of integer = { 0 };

lock(integer i) {
    Entering[i] = true;
    Number[i] = 1 + max(Number[1], ..., Number[NUM_THREADS]);
    Entering[i] = false;
    for (integer j = 1; j <= NUM_THREADS; j++) {
        // 스레드 j가 자신의 번호를 받을 때까지 대기한다.
        while (Entering[j]);
        // 숫자가 더 작거나 같은 스레드가 모두 나올 때까지 기다립니다.
        // 같은 숫자이지만 우선 순위가 더 높으면 작업을 완료합니다.
        while ((Number[j] != 0) && (Number[j], j) < (Number[i], i));
    }
}

unlock(integer i) {
    Number[i] = 0;
}
```

</br>
</br>

## 직접 구현해 보기
'다카노 유키 - 동시성 프로그래밍'책을 참고하여 Rust로 알고리즘을 구현해 보았다.

</br>

#### examples/algorithm_ex_0.rs
```Rust
...

// 컴파일 최적화를 방지하기 위해 추가함.
use std::ptr::{read_volatile, write_volatile};

unsafe fn bakery_lock_acq(idx: usize) {
    // Entering[i] = true;
    write_volatile(&mut ENTERING[idx], true);

    // Number[i] = 1 + max(Number[1], ..., Number[NUM_THREADS]);
    let ticket = 1 + TICKETS.iter()
        .filter_map(|&ticket| ticket)
        .max().unwrap_or(0);
    write_volatile(&mut TICKETS[idx], Some(ticket));

    // Entering[i] = false;
    write_volatile(&mut ENTERING[idx], false);

    for i in 0..NUM_THREADS {
        if i == idx { 
            continue;
        }

        // 스레드 j가 자신의 번호를 받을 때까지 대기한다.
        while read_volatile(&ENTERING[i]) { }

        // 숫자가 더 작거나 같은 스레드가 모두 나올 때까지 기다립니다.
        // 같은 숫자이지만 우선 순위가 더 높으면 작업을 완료합니다.
        loop {
            if let Some(t) = read_volatile(&TICKETS[i]) {
                if ticket < t || (ticket == t && idx < i) {
                    break;
                }
            }
            else {
                break;
            }
        }
    }
}

...
```

> 실행결과: SUM=7838743, (expected=8000000) </br>
(실행 할때마다 결과가 다르게 나온다.)

</br>

안타깝게도 이상한 결과 값이 나온다. 
이런 결과가 나온 이유는 바로 현대 CPU는 명령 실행 효율을 높이기 위해 '비순차적 실행(out-of-order execution)'을 사용하기 때문이다. 
따라서 메모리 접근이 반드시 명령어 순으로 실행되지는 않는다. </br>
올바르게 작동되게 하려면 메모리 접근의 작동 순서를 보증하기 위한 명령을 이용해야 한다.

</br>

#### examples/algorithm_ex_1.rs
```Rust
...

unsafe fn bakery_lock_acq(idx: usize) {
    // Entering[i] = true;
    fence(Ordering::SeqCst);
    write_volatile(&mut ENTERING[idx], true);
    fence(Ordering::SeqCst);

    // Number[i] = 1 + max(Number[1], ..., Number[NUM_THREADS]);
    let ticket = 1 + TICKETS.iter()
        .filter_map(|&ticket| ticket)
        .max().unwrap_or(0);
    write_volatile(&mut TICKETS[idx], Some(ticket));

    // Entering[i] = false;
    fence(Ordering::SeqCst);
    write_volatile(&mut ENTERING[idx], false);
    fence(Ordering::SeqCst);

    for i in 0..NUM_THREADS {
        if i == idx { 
            continue;
        }

        // 스레드 j가 자신의 번호를 받을 때까지 대기한다.
        while read_volatile(&ENTERING[i]) { }

        // 숫자가 더 작거나 같은 스레드가 모두 나올 때까지 기다립니다.
        // 같은 숫자이지만 우선 순위가 더 높으면 작업을 완료합니다.
        loop {
            if let Some(t) = read_volatile(&TICKETS[i]) {
                if ticket < t || (ticket == t && idx < i) {
                    break;
                }
            }
            else {
                break;
            }
        }
    }

    fence(Ordering::SeqCst);
}
...
```
> 실행결과: SUM=8000000, (expected=8000000) </br>
