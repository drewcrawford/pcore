fn main() {
    windows::build! {
        Windows::Win32::Foundation::PWSTR,
        Windows::Win32::System::Diagnostics::Debug::WIN32_ERROR
    }
}