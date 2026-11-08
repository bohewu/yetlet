//! Styles-only NTFS replacement, using the existing pinned native boundary.
use super::*;
pub(crate) fn replace(
    app: &App,
    key: &OsStr,
    old: Option<&[u8]>,
    data: &[u8],
    replaced: &mut bool,
) -> io::Result<()> {
    let check = || match old {
        Some(bytes) if static_output::read(app, key, 2 * 1024 * 1024)? == bytes => Ok(()),
        Some(_) => Err(io::ErrorKind::InvalidData.into()),
        None => static_output::absent(app, key),
    };
    check()?;
    if old == Some(data) {
        return Ok(());
    }
    let tmp = format!(".yetlet-style-{}", std::process::id());
    let sd = private_security()?;
    let mut file = relative(app.parent.directory(), tmp.as_ref(), false, true, Some(&sd))?;
    let result = (|| {
        file.write_all(data)?;
        file.sync_all()?;
        check()?;
        let name = component(key)?;
        let length = (offset_of!(FILE_RENAME_INFORMATION, FileName) + name.len() * 2)
            .max(size_of::<FILE_RENAME_INFORMATION>());
        let mut storage = vec![0_usize; length.div_ceil(size_of::<usize>())];
        let info = storage.as_mut_ptr().cast::<FILE_RENAME_INFORMATION>();
        unsafe {
            (*info).Anonymous.ReplaceIfExists = old.is_some();
            (*info).RootDirectory = handle(app.parent.directory());
            (*info).FileNameLength = (name.len() * 2) as u32;
            std::ptr::copy_nonoverlapping(name.as_ptr(), (*info).FileName.as_mut_ptr(), name.len());
        }
        let mut status = IO_STATUS_BLOCK::default();
        nt(unsafe {
            NtSetInformationFile(
                handle(&file),
                &mut status,
                info.cast(),
                length as u32,
                FileRenameInformation,
            )
        })?;
        *replaced = true;
        #[cfg(test)]
        crate::platform::fs::styles_output::after_rename(key)?;
        app.verify()
    })();
    if result.is_err() {
        if let Ok(current) = relative(app.parent.directory(), tmp.as_ref(), false, false, None) {
            if inspect(&current, false)? == inspect(&file, false)? {
                let _ = remove(&file);
            }
        }
    }
    result
}
