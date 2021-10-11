fn main() {
    windows::build! {
        Windows::Win32::System::Diagnostics::Debug::{WIN32_ERROR,GetLastError},
        Windows::Win32::System::WinRT::{HSTRING_HEADER,WindowsCreateStringReference},
        //used only in tests
        Windows::Foundation::Uri,
    }
}