#![no_std]
use near_sys as sys;

#[panic_handler]
fn on_panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { sys::panic() }
}

fn panic_msg(msg: &[u8]) -> ! {
    unsafe { sys::panic_utf8(msg.len() as u64, msg.as_ptr() as u64) }
}

fn read_id(reg: u64, buf: &mut [u8]) -> usize {
    let len = unsafe { sys::register_len(reg) };
    if len as usize > buf.len() {
        panic_msg(b"too long");
    }
    unsafe { sys::read_register(reg, buf.as_ptr() as u64) };
    len as usize
}

fn find_value(input: &[u8], key: &[u8], buf: &mut [u8]) -> usize {
    let mut i = 0;
    while i < input.len() {
        if i + key.len() + 2 <= input.len()
            && input[i] == b'"'
            && &input[i + 1..i + 1 + key.len()] == key
            && input[i + 1 + key.len()] == b'"'
        {
            let mut cursor = i + key.len() + 2;
            while cursor < input.len()
                && (input[cursor] == b' '
                    || input[cursor] == b'\t'
                    || input[cursor] == b'\r'
                    || input[cursor] == b'\n')
            {
                cursor += 1;
            }
            if cursor >= input.len() || input[cursor] != b':' {
                i += 1;
                continue;
            }
            cursor += 1;
            while cursor < input.len()
                && (input[cursor] == b' '
                    || input[cursor] == b'\t'
                    || input[cursor] == b'\r'
                    || input[cursor] == b'\n')
            {
                cursor += 1;
            }
            if cursor >= input.len() || input[cursor] != b'"' {
                panic_msg(b"no quote");
            }
            cursor += 1;
            let start = cursor;
            while cursor < input.len() && input[cursor] != b'"' {
                if input[cursor] == b'\\' {
                    panic_msg(b"backslash");
                }
                cursor += 1;
            }
            if cursor >= input.len() {
                panic_msg(b"no end quote");
            }
            let len = cursor - start;
            if len > buf.len() {
                panic_msg(b"val too long");
            }
            buf[..len].copy_from_slice(&input[start..cursor]);
            return len;
        }
        i += 1;
    }
    panic_msg(b"key missing")
}

fn parse_u128(bytes: &[u8]) -> u128 {
    if bytes.is_empty() {
        panic_msg(b"empty amt");
    }
    let mut res: u128 = 0;
    for &b in bytes {
        if !b.is_ascii_digit() {
            panic_msg(b"non digit");
        }
        res = res
            .checked_mul(10)
            .ok_or(())
            .and_then(|r| r.checked_add((b - b'0') as u128).ok_or(()))
            .unwrap_or_else(|_| panic_msg(b"overflow"));
    }
    res
}

fn verify_parent() {
    let mut buf_cur = [0u8; 64];
    unsafe { sys::current_account_id(0) };
    let cur_len = read_id(0, &mut buf_cur);
    let cur = &buf_cur[..cur_len];
    let mut dot_pos = None;
    for (i, &b) in cur.iter().enumerate() {
        if b == b'.' {
            dot_pos = Some(i);
            break;
        }
    }
    let start = match dot_pos {
        Some(p) => p + 1,
        None => panic_msg(b"no dot"),
    };
    let parent = &cur[start..];
    let mut buf_pred = [0u8; 64];
    unsafe { sys::predecessor_account_id(0) };
    let pred_len = read_id(0, &mut buf_pred);
    let pred = &buf_pred[..pred_len];
    if parent != pred {
        panic_msg(b"not parent");
    }
}

#[no_mangle]
pub extern "C" fn ft_send() {
    verify_parent();
    let mut input_buf = [0u8; 512];
    unsafe { sys::input(0) };
    let input_len = read_id(0, &mut input_buf);
    let input = &input_buf[..input_len];

    let mut token = [0u8; 64];
    let token_len = find_value(input, b"token", &mut token);
    let mut receiver = [0u8; 64];
    let receiver_len = find_value(input, b"receiver", &mut receiver);
    let mut amount = [0u8; 64];
    let amount_len = find_value(input, b"amount", &mut amount);
    parse_u128(&amount[..amount_len]);

    let mut arg_buf = [0u8; 256];
    let mut arg_ptr = 0;

    let prefix = b"{\"receiver_id\":\"";
    arg_buf[..prefix.len()].copy_from_slice(prefix);
    arg_ptr += prefix.len();
    arg_buf[arg_ptr..arg_ptr + receiver_len].copy_from_slice(&receiver[..receiver_len]);
    arg_ptr += receiver_len;
    let mid = b"\",\"amount\":\"";
    arg_buf[arg_ptr..arg_ptr + mid.len()].copy_from_slice(mid);
    arg_ptr += mid.len();
    arg_buf[arg_ptr..arg_ptr + amount_len].copy_from_slice(&amount[..amount_len]);
    arg_ptr += amount_len;
    let suffix = b"\"}";
    arg_buf[arg_ptr..arg_ptr + suffix.len()].copy_from_slice(suffix);
    arg_ptr += suffix.len();

    let one_yocto: u128 = 1;
    unsafe {
        sys::promise_batch_create(token_len as u64, token.as_ptr() as u64);
        sys::promise_batch_action_function_call(
            0,
            11,
            b"ft_transfer".as_ptr() as u64,
            arg_ptr as u64,
            arg_buf.as_ptr() as u64,
            &one_yocto as *const u128 as u64,
            30_000_000_000_000,
        );
        sys::promise_return(0);
    }
}

#[no_mangle]
pub extern "C" fn near_send() {
    verify_parent();
    let mut input_buf = [0u8; 512];
    unsafe { sys::input(0) };
    let input_len = read_id(0, &mut input_buf);
    let input = &input_buf[..input_len];

    let mut receiver = [0u8; 64];
    let receiver_len = find_value(input, b"receiver", &mut receiver);
    let mut amount_bytes = [0u8; 64];
    let amount_len = find_value(input, b"amount", &mut amount_bytes);
    let amount = parse_u128(&amount_bytes[..amount_len]);

    unsafe {
        sys::promise_batch_create(receiver_len as u64, receiver.as_ptr() as u64);
        sys::promise_batch_action_transfer(0, &amount as *const u128 as u64);
        sys::promise_return(0);
    }
}

#[no_mangle]
pub extern "C" fn ft_on_transfer() {
    let val = b"\"0\"";
    unsafe { sys::value_return(val.len() as u64, val.as_ptr() as u64) };
}

#[no_mangle]
pub extern "C" fn nft_on_transfer() {
    let val = b"true";
    unsafe { sys::value_return(val.len() as u64, val.as_ptr() as u64) };
}
