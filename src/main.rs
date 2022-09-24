use std::{
    collections::HashMap,
    mem::{size_of, zeroed, MaybeUninit},
    ptr::null_mut,
    thread,
};

use winapi::{
    shared::{
        minwindef::{HINSTANCE, LPARAM, LRESULT, WPARAM},
        windef::HHOOK__,
    },
    um::{
        winnt::POWER_ACTION_LOCK_CONSOLE,
        winuser::{
            CallNextHookEx, GetAsyncKeyState, GetMessageW, INPUT_u, MapVirtualKeyW, SendInput,
            SetWindowsHookExW, UnhookWindowsHookEx, INPUT, INPUT_KEYBOARD, KBDLLHOOKSTRUCT,
            KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, LPINPUT,
            VK_BACK, VK_CAPITAL, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_F1, VK_F10, VK_F11,
            VK_F12, VK_F2, VK_F22, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_HOME,
            VK_INSERT, VK_LCONTROL, VK_LEFT, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MENU, VK_NEXT,
            VK_OEM_1, VK_OEM_2, VK_OEM_3, VK_OEM_4, VK_PRIOR, VK_RETURN, VK_RIGHT, VK_RMENU,
            VK_SPACE, VK_UP, WH_KEYBOARD_LL, WM_KEYUP,
        },
    },
};

// use winput::Vk;

fn main() {
    let _handle = thread::Builder::new()
        .name("rusty_keyboard".into())
        .spawn(|| {
            let cleaner = run_keyboard_interceptor();

            let mut msg = unsafe { MaybeUninit::zeroed().assume_init() };
            unsafe {
                GetMessageW(&mut msg, null_mut(), 0, 0);
            };
            println!("{}", msg.message);
        });

    loop {}
}

fn run_keyboard_interceptor() -> CleanUpHookStruct {
    let hook: *mut HHOOK__;
    unsafe {
        println!("setting hook");
        hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(callback), 0 as HINSTANCE, 0);
        println!("installed hook {:?}", hook)
    }
    CleanUpHookStruct { hook }
}

struct CleanUpHookStruct {
    hook: *mut HHOOK__,
}
impl CleanUpHookStruct {
    fn un_hook(&self) {
        println!("cleaning hook {:?}", self.hook);
        unsafe {
            UnhookWindowsHookEx(self.hook);
        }
    }
}

struct ExtensionMap {
    key_map: HashMap<i32, Vec<i32>>,
}
impl ExtensionMap {
    fn new() -> ExtensionMap {
        let mut key_map = HashMap::new();

        //top line off keyboard a->p
        key_map.insert(VK_A, vec![VK_ESCAPE]);
        // self.keyMap.insert(VK_Z, VK_BACK);//  I think browser backwards
        //self.keyMap.insert(VK_E, VK_E); //ctrl + f
        //self.keyMap.insert(VK_R, VK_R); //  I think browser forward
        key_map.insert(VK_T, vec![VK_INSERT]);
        key_map.insert(VK_Y, vec![VK_PRIOR]); //page up
        key_map.insert(VK_U, vec![VK_HOME]);
        key_map.insert(VK_I, vec![VK_UP]);
        key_map.insert(VK_O, vec![VK_END]);
        // self.keyMap.insert(VK_P, VK_P);//??? right click menu? shift + 10

        //middle line off keyboard q->m
        key_map.insert(VK_Q, vec![VK_LMENU]);
        key_map.insert(VK_S, vec![VK_LWIN]);
        key_map.insert(VK_D, vec![VK_LSHIFT]);
        key_map.insert(VK_F, vec![VK_LCONTROL]);
        key_map.insert(VK_G, vec![VK_RMENU]);
        key_map.insert(VK_H, vec![VK_NEXT]); //page down
        key_map.insert(VK_J, vec![VK_LEFT]);
        key_map.insert(VK_K, vec![VK_DOWN]);
        key_map.insert(VK_L, vec![VK_RIGHT]);
        key_map.insert(VK_M, vec![VK_DELETE]);

        //bottom line of keyboard
        // self.keyMap.insert(VK_W, VK_ESCAPE);//crtl + z
        // self.keyMap.insert(VK_X, VK_ESCAPE);ctrl + shift + z
        // self.keyMap.insert(VK_C, VK_ESCAPE); // ctrl + x
        // self.keyMap.insert(VK_V, VK_ESCAPE);// ctrl + c
        // self.keyMap.insert(VK_B, VK_ESCAPE); // ctrl + v
        // self.keyMap.insert(VK_N, VK_ESCAPE); //empty
        key_map.insert(VK_OEM_1, vec![VK_BACK]); //print?
                                                 // key_map.insert(VK_OEM_2, vec![VK_BACK]);
                                                 // self.keyMap.insert(VK_OEM_3, VK_ESCAPE);//???
                                                 // self.keyMap.insert(VK_OEM_4, VK_ESCAPE);//win key

        key_map.insert(VK_SPACE, vec![VK_RETURN]); //enter

        ExtensionMap { key_map }
    }
}

unsafe extern "system" fn callback(
    n_code: libc::c_int,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    let key_map = ExtensionMap::new();
    let hook_struct = l_param as *mut KBDLLHOOKSTRUCT;
    let vk: i32 = (*hook_struct)
        .vkCode
        .try_into()
        .expect("vk doesn't fit in i32");

    let is_release = w_param == WM_KEYUP.try_into().unwrap();
    //most significant bit tells you if the key is being pressed
    //clean output by bitwise and-ing and comparing to the exact bit representation
    let modifier_active =
        ((GetAsyncKeyState(0x85) as u16) & 0b_1000_0000_0000_0000) == 0b_1000_0000_0000_0000;

    //remap capslock
    if vk == VK_CAPITAL
    /*20 */
    {
        //to f22
        println!("Setting modifier {}", !is_release);
        send_key(VK_F22 /*133 */, is_release);
        return 1;
    }

    if modifier_active && key_map.key_map.contains_key(&vk) {
        let output_keys = key_map.key_map.get(&vk).expect("key in dictionary");
        let first_key_in_output = output_keys.first().expect("should be at least one key");
        println!("Rebinding {:#0x} to {:#0x}", vk, first_key_in_output);
        send_key(*first_key_in_output, is_release);
        return 1;
    }
    // //if key == "i"
    // if vk == 0x49 && modifier_active {
    //     //send vk_up
    //     send_key(0x26, is_release);
    //     return 1;
    // }
    // //if key == "k"
    // if vk == 0x4B && modifier_active {
    //     //send vk_down
    //     send_key(0x28, is_release);
    //     return 1;
    // }

    // //if key = D
    // if vk == 0x44 && modifier_active {
    //     //send shift
    //     send_key(0xA0, is_release);
    //     return 1;
    // }

    CallNextHookEx(null_mut(), n_code, w_param, l_param)
}

unsafe fn send_key(key: i32, is_release: bool) {
    let mut input_u: INPUT_u = zeroed();

    *input_u.ki_mut() = KEYBDINPUT {
        wVk: key.try_into().unwrap(),
        dwExtraInfo: 0,
        // //set flag to check scan code instead of wvk
        // //bit OR with up event if the original was a up event
        // dwFlags: KEYEVENTF_SCANCODE | if is_release { KEYEVENTF_KEYUP } else { 0 },
        dwFlags: KEYEVENTF_EXTENDEDKEY | if is_release { KEYEVENTF_KEYUP } else { 0 },
        time: 0,
        //send VK_UP
        wScan: MapVirtualKeyW(key.try_into().unwrap(), 0)
            .try_into()
            .expect("mapping vk to scan failed"),
    };
    let mut input = INPUT {
        type_: INPUT_KEYBOARD,
        u: input_u,
    };
    let ipsize = size_of::<INPUT>() as i32;
    SendInput(1, &mut input, ipsize);
}

const VK_0: i32 = 0x30;
const VK_1: i32 = 0x31;
const VK_2: i32 = 0x32;
const VK_3: i32 = 0x33;
const VK_4: i32 = 0x34;
const VK_5: i32 = 0x35;
const VK_6: i32 = 0x36;
const VK_7: i32 = 0x37;
const VK_8: i32 = 0x38;
const VK_9: i32 = 0x39;
const VK_A: i32 = 0x41;
const VK_B: i32 = 0x42;
const VK_C: i32 = 0x43;
const VK_D: i32 = 0x44;
const VK_E: i32 = 0x45;
const VK_F: i32 = 0x46;
const VK_G: i32 = 0x47;
const VK_H: i32 = 0x48;
const VK_I: i32 = 0x49;
const VK_J: i32 = 0x4A;
const VK_K: i32 = 0x4B;
const VK_L: i32 = 0x4C;
const VK_M: i32 = 0x4D;
const VK_N: i32 = 0x4E;
const VK_O: i32 = 0x4F;
const VK_P: i32 = 0x50;
const VK_Q: i32 = 0x51;
const VK_R: i32 = 0x52;
const VK_S: i32 = 0x53;
const VK_T: i32 = 0x54;
const VK_U: i32 = 0x55;
const VK_V: i32 = 0x56;
const VK_W: i32 = 0x57;
const VK_X: i32 = 0x58;
const VK_Y: i32 = 0x59;
const VK_Z: i32 = 0x5A;
