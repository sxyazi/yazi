use std::cell::RefCell;

use windows::{Win32::{Foundation::{E_ABORT, E_FAIL}, UI::Shell::*}, core::*};

#[implement(IFileOperationProgressSink, Agile = false)]
pub(super) struct RecycleSink(pub(super) RefCell<Result<IShellItem>>);

impl Default for RecycleSink {
	fn default() -> Self {
		Self(RefCell::new(Err(Error::new(E_FAIL, "No recycled item was returned"))))
	}
}

impl IFileOperationProgressSink_Impl for RecycleSink_Impl {
	fn StartOperations(&self) -> Result<()> { Ok(()) }

	fn FinishOperations(&self, _hrresult: HRESULT) -> Result<()> { Ok(()) }

	fn PreRenameItem(
		&self,
		_dwflags: u32,
		_psiitem: Ref<IShellItem>,
		_psznewname: &PCWSTR,
	) -> Result<()> {
		Ok(())
	}

	fn PostRenameItem(
		&self,
		_dwflags: u32,
		_psiitem: Ref<IShellItem>,
		_psznewname: &PCWSTR,
		_hrrename: HRESULT,
		_psinewlycreated: Ref<IShellItem>,
	) -> Result<()> {
		Ok(())
	}

	fn PreMoveItem(
		&self,
		_dwflags: u32,
		_psiitem: Ref<IShellItem>,
		_psidestinationfolder: Ref<IShellItem>,
		_psznewname: &PCWSTR,
	) -> Result<()> {
		Ok(())
	}

	fn PostMoveItem(
		&self,
		_dwflags: u32,
		_psiitem: Ref<IShellItem>,
		_psidestinationfolder: Ref<IShellItem>,
		_psznewname: &PCWSTR,
		_hrmove: HRESULT,
		_psinewlycreated: Ref<IShellItem>,
	) -> Result<()> {
		Ok(())
	}

	fn PreCopyItem(
		&self,
		_dwflags: u32,
		_psiitem: Ref<IShellItem>,
		_psidestinationfolder: Ref<IShellItem>,
		_psznewname: &PCWSTR,
	) -> Result<()> {
		Ok(())
	}

	fn PostCopyItem(
		&self,
		_dwflags: u32,
		_psiitem: Ref<IShellItem>,
		_psidestinationfolder: Ref<IShellItem>,
		_psznewname: &PCWSTR,
		_hrcopy: HRESULT,
		_psinewlycreated: Ref<IShellItem>,
	) -> Result<()> {
		Ok(())
	}

	fn PreDeleteItem(&self, flags: u32, _psiitem: Ref<IShellItem>) -> Result<()> {
		if flags & TSF_DELETE_RECYCLE_IF_POSSIBLE.0 as u32 == 0 {
			Err(Error::from_hresult(E_ABORT))
		} else {
			Ok(())
		}
	}

	fn PostDeleteItem(
		&self,
		_dwflags: u32,
		_psiitem: Ref<IShellItem>,
		hrdelete: HRESULT,
		created: Ref<IShellItem>,
	) -> Result<()> {
		*self.0.borrow_mut() = hrdelete.ok().and_then(|()| {
			created.cloned().ok_or_else(|| Error::new(E_FAIL, "No recycled item was returned"))
		});
		Ok(())
	}

	fn PreNewItem(
		&self,
		_dwflags: u32,
		_psidestinationfolder: Ref<IShellItem>,
		_psznewname: &PCWSTR,
	) -> Result<()> {
		Ok(())
	}

	fn PostNewItem(
		&self,
		_dwflags: u32,
		_psidestinationfolder: Ref<IShellItem>,
		_psznewname: &PCWSTR,
		_psztemplatename: &PCWSTR,
		_dwfileattributes: u32,
		_hrnew: HRESULT,
		_psinewitem: Ref<IShellItem>,
	) -> Result<()> {
		Ok(())
	}

	fn UpdateProgress(&self, _iworktotal: u32, _iworksofar: u32) -> Result<()> { Ok(()) }

	fn ResetTimer(&self) -> Result<()> { Ok(()) }

	fn PauseTimer(&self) -> Result<()> { Ok(()) }

	fn ResumeTimer(&self) -> Result<()> { Ok(()) }
}
