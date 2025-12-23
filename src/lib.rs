use std::ffi::{c_int};
use std::mem::size_of;
use pgrx::prelude::*;
use pgrx::prelude::PgTryBuilder;


::pgrx::pg_module_magic!(name, version);

// --- PostgreSQL-derived code ---
// Portions Copyright (c) 1996-2025, PostgreSQL Global Development Group
// Portions Copyright (c) 1994, The Regents of the University of California
//
// Permission to use, copy, modify, and distribute this software and its
// documentation for any purpose, without fee, and without a written agreement
// is hereby granted, provided that the above copyright notice and this
// paragraph and the following two paragraphs appear in all copies.
//
// IN NO EVENT SHALL THE UNIVERSITY OF CALIFORNIA BE LIABLE TO ANY PARTY FOR
// DIRECT, INDIRECT, SPECIAL, INCIDENTAL, OR CONSEQUENTIAL DAMAGES, INCLUDING
// LOST PROFITS, ARISING OUT OF THE USE OF THIS SOFTWARE AND ITS
// DOCUMENTATION, EVEN IF THE UNIVERSITY OF CALIFORNIA HAS BEEN ADVISED OF THE
// POSSIBILITY OF SUCH DAMAGE.
//
// THE UNIVERSITY OF CALIFORNIA SPECIFICALLY DISCLAIMS ANY WARRANTIES,
// INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY
// AND FITNESS FOR A PARTICULAR PURPOSE.  THE SOFTWARE PROVIDED HEREUNDER IS
// ON AN "AS IS" BASIS, AND THE UNIVERSITY OF CALIFORNIA HAS NO OBLIGATIONS TO
// PROVIDE MAINTENANCE, SUPPORT, UPDATES, ENHANCEMENTS, OR MODIFICATIONS.
//
// Source: PostgreSQL src/backend/commands/sequence.c
// Source: PostgreSQL src/include/commands/sequence.h
// Source: PostgreSQL src/include/c.h
//
// Changes: adapted to Rust and added grab_advisory_lock() calls where applicable in accordance with
// the objectives of this project

// struct SeqTableData
// {
// 	Oid			relid;			/* pg_class OID of this sequence (hash key) */
// 	RelFileNumber filenumber;	/* last seen relfilenumber of this sequence */
// 	LocalTransactionId lxid;	/* xact in which we last did a seq op */
// 	bool		last_valid;		/* do we have a valid "last" value? */
// 	int64		last;			/* value last returned by nextval */
// 	int64		cached;			/* last value already cached for nextval */
// 	/* if last != cached, we have not used up all the cached values */
// 	int64		increment;		/* copy of sequence's increment field */
// 	/* note that increment is zero until we first do nextval_internal() */
// } SeqTableData;
#[repr(C)]
struct SeqTableData {
    relid: pg_sys::Oid,
    filenumber: pg_sys::Oid,
    lxid: u32,
    last_valid: bool,
    last: i64,
    cached: i64,
    increment: i64,
}
type SeqTable = *mut SeqTableData;

// typedef struct FormData_pg_sequence_data
// {
// 	int64		last_value;
// 	int64		log_cnt;
// 	bool		is_called;
// } FormData_pg_sequence_data;
//
// typedef FormData_pg_sequence_data *Form_pg_sequence_data;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FormData_pg_sequence {
    pub seqrelid: pg_sys::Oid,
    pub seqtypid: pg_sys::Oid,
    pub seqstart: i64,
    pub seqincrement: i64,
    pub seqmax: i64,
    pub seqmin: i64,
    pub seqcache: i64,
    pub seqcycle: bool,
}
impl Default for FormData_pg_sequence {
    fn default() -> Self {
        let mut s = ::core::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::core::ptr::write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}
#[allow(non_camel_case_types)]
pub type Form_pg_sequence = *mut FormData_pg_sequence;

// typedef struct sequence_magic
// {
// 	uint32		magic;
// } sequence_magic;
#[repr(C)]
struct SequenceMagic {
    magic: u32
}

// /*
//  * The "special area" of a sequence's buffer page looks like this.
//  */
// #define SEQ_MAGIC		0x1717
const SEQ_MAGIC: u32 = 0x1717;

// typedef struct FormData_pg_sequence_data
// {
// 	int64		last_value;
// 	int64		log_cnt;
// 	bool		is_called;
// } FormData_pg_sequence_data;
//
// typedef FormData_pg_sequence_data *Form_pg_sequence_data;

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct FormData_pg_sequence_data {
    last_value: i64,
    log_cnt: i64,
    is_called: bool,
}
#[allow(non_camel_case_types)]
pub type Form_pg_sequence_data = *mut FormData_pg_sequence_data;

// /*
//  * We don't want to log each fetching of a value from a sequence,
//  * so we pre-log a few fetches in advance. In the event of
//  * crash we can lose (skip over) as many values as we pre-logged.
//  */
// #define SEQ_LOG_VALS	32
const SEQ_LOG_VALS: i64 = 32;

// #define InvalidSubTransactionId		((SubTransactionId) 0)
#[allow(non_upper_case_globals)]
const InvalidSubTransactionId: u32 = 0;

// /* Sequence WAL record */
// typedef struct xl_seq_rec
// {
// 	RelFileLocator locator;
// 	/* SEQUENCE TUPLE DATA FOLLOWS AT THE END */
// } xl_seq_rec;
#[repr(C)]
struct XlSeqRec {
    locator: pg_sys::RelFileLocator,
}
impl Default for XlSeqRec {
    fn default() -> Self {
        let mut s = ::core::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::core::ptr::write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}

// /* Record identifier */
// #define XLOG_SEQ_LOG			0x00
const XLOG_SEQ_LOG: u8 = 0x00;

// Backend is single-threaded, so it is safe to use a raw static pointer
// Safety: PostgreSQL dynahash tables live in a global memory context and are
// only accessed inside the backend's single-threaded execution., thus it is safe to use a raw
// static pointer. See also get_seq_hashtab()
#[allow(non_camel_case_types)]
static mut SEQHASHTAB: *mut pg_sys::HTAB = std::ptr::null_mut();

// Helper to lazily initialize the sequence hash table in TopMemoryContext
unsafe fn get_seq_hashtab() -> *mut pg_sys::HTAB {
    if SEQHASHTAB.is_null() {
        let mut ctl: pg_sys::HASHCTL = pg_sys::HASHCTL::default();
        ctl.keysize = size_of::<pg_sys::Oid>();
        ctl.entrysize = size_of::<SeqTableData>();

        // Switch to TopMemoryContext so the table lives for backend lifetime
        let old_ctx = pg_sys::MemoryContextSwitchTo(pg_sys::TopMemoryContext);
        SEQHASHTAB = pg_sys::hash_create(
            b"Sequence values\0".as_ptr().cast(),
            16,
            &mut ctl,
            (pg_sys::HASH_ELEM | pg_sys::HASH_BLOBS) as c_int,
        );
        pg_sys::MemoryContextSwitchTo(old_ctx);
    }
    SEQHASHTAB
}

unsafe fn nextval_with_xact_lock_internal(relid: pg_sys::Oid, check_permissions: bool) -> i64 {
    let mut elm: SeqTable = SeqTable::default();
    let mut seqrel: pg_sys::Relation = pg_sys::Relation::default();

    init_sequence(relid, &mut elm, &mut seqrel);

    if check_permissions &&
        pg_sys::pg_class_aclcheck(
            (*elm).relid,
            pg_sys::GetUserId(),
            (pg_sys::ACL_USAGE | pg_sys::ACL_UPDATE) as pg_sys::AclMode
        ) != pg_sys::AclResult::ACLCHECK_OK {
        pg_sys::ereport!(
            pg_sys::elog::PgLogLevel::ERROR,
            pg_sys::errcodes::PgSqlErrorCode::ERRCODE_INSUFFICIENT_PRIVILEGE,
            format!("permission denied for sequence {:?}", (*(*seqrel).rd_rel).relname),
        );
    }

    if !(*seqrel).rd_islocaltemp {
        pg_sys::PreventCommandIfReadOnly(b"nextval_with_xact_lock()\0".as_ptr().cast());
    }

    // /*
    //  * Forbid this during parallel operation because, to make it work, the
    //  * cooperating backends would need to share the backend-local cached
    //  * sequence information.  Currently, we don't support that.
    //  */
    pg_sys::PreventCommandIfParallelMode(b"nextval_with_xact_lock()\0".as_ptr().cast());

    if (*elm).last != (*elm).cached {
        assert!((*elm).last_valid);
        assert_ne!((*elm).increment, 0);
        (*elm).last += (*elm).increment;

        PgTryBuilder::new(|| {
            grab_advisory_lock((*elm).last);
        }).catch_others(|error| {
            sequence_close(seqrel, pg_sys::NoLock);
            error.rethrow();
        }).execute();

        sequence_close(seqrel, pg_sys::NoLock);
        return (*elm).last;
    }
    let pgstuple = pg_sys::SearchSysCache1(
        pg_sys::SysCacheIdentifier::SEQRELID as c_int, pg_sys::ObjectIdGetDatum(relid)
    );
    if pgstuple.is_null() {
        pg_sys::ereport!(
            pg_sys::elog::PgLogLevel::ERROR,
            pg_sys::errcodes::PgSqlErrorCode::ERRCODE_UNDEFINED_OBJECT,
            format!("cache lookup failed for sequence {:?}", relid),
        );
    }
    let pgsform = pg_sys::GETSTRUCT(pgstuple) as Form_pg_sequence;
    let incby = (*pgsform).seqincrement;
    let maxv = (*pgsform).seqmax;
    let minv = (*pgsform).seqmin;
    let cache = (*pgsform).seqcache;
    let cycle = (*pgsform).seqcycle;
    let mut buf: pg_sys::Buffer = pg_sys::Buffer::default();
    let mut seqdatatuple: pg_sys::HeapTupleData = pg_sys::HeapTupleData::default();
    pg_sys::ReleaseSysCache(pgstuple);

    // /* lock page buffer and read tuple */
    let seq = read_seq_tuple(seqrel, &mut buf, &mut seqdatatuple);
    let page = pg_sys::BufferGetPage(buf);
    let mut last: i64;
    let mut next: i64;
    let mut result: i64;
    let mut rescnt: i64 = 0;

    last = (*seq).last_value;
    next = (*seq).last_value;
    result = (*seq).last_value;
    let mut fetch = cache;
    let mut log = (*seq).log_cnt;
    let mut logit: bool = false;
    if !(*seq).is_called {
        rescnt += 1;
        fetch -= 1;
    }

    // /*
    // 	 * Decide whether we should emit a WAL log record.  If so, force up the
    // 	 * fetch count to grab SEQ_LOG_VALS more values than we actually need to
    // 	 * cache.  (These will then be usable without logging.)
    // 	 *
    // 	 * If this is the first nextval after a checkpoint, we must force a new
    // 	 * WAL record to be written anyway, else replay starting from the
    // 	 * checkpoint would fail to advance the sequence past the logged values.
    // 	 * In this case we may as well fetch extra values.
    // 	 */

    if (log < fetch) || !(*seq).is_called {
        fetch = fetch + SEQ_LOG_VALS;
        log = fetch;
        logit = true;
    } else {
        let redoptr = pg_sys::GetRedoRecPtr();

        if pg_sys::PageGetLSN(page) <= redoptr {
            /* last update of seq was before checkpoint */
            fetch = fetch + SEQ_LOG_VALS;
            log = fetch;
            logit = true;
        }
    }

    while fetch != 0 {
        if incby > 0 {
            if (maxv >= 0 && next > maxv - incby) || (maxv < 0 && next + incby > maxv) {
                if rescnt > 0 {
                    break;
                }
                if !cycle {
                    pg_sys::ereport!(
                        pg_sys::elog::PgLogLevel::ERROR,
                        pg_sys::errcodes::PgSqlErrorCode::ERRCODE_SEQUENCE_GENERATOR_LIMIT_EXCEEDED,
                        format!(
                            "nextval_with_xact_lock: reached maximum value of sequence {:?} ({})",
                            (*(*seqrel).rd_rel).relname,
                            maxv
                        ),
                    );
                }
                next = minv;
            } else {
                next += incby;
            }
        } else {
            /* descending sequence */
            if (minv < 0 && next < minv - incby) || (minv >= 0 && next + incby < minv) {
                if rescnt > 0 {
                    break;
                }

                if !cycle {
                    pg_sys::ereport!(
                        pg_sys::elog::PgLogLevel::ERROR,
                        pg_sys::errcodes::PgSqlErrorCode::ERRCODE_SEQUENCE_GENERATOR_LIMIT_EXCEEDED,
                        format!(
                            "nextval_with_xact_lock: reached minimum value of sequence {:?} ({})",
                            (*(*seqrel).rd_rel).relname,
                            minv
                        ),
                    );
                }
                next = maxv;
            } else {
                next += incby;
            }
        }
        fetch = fetch - 1;
        if rescnt < cache {
            log -= 1;
            rescnt += 1;
            last = next;
            // /* if it's first result - */
            if rescnt == 1 {
                // /* it's what to return */
                result = next;
            }
        }
    }
    // /* adjust for any unfetched numbers */
    log -= fetch;
    assert!(log >= 0);

    // /* save info in local cache */
    (*elm).increment = incby;
    (*elm).last = result; // /* last returned number */
    (*elm).cached = last; // /* last fetched number */
    (*elm).last_valid = true;

    // /*
    // 	 * If something needs to be WAL logged, acquire an xid, so this
    // 	 * transaction's commit will trigger a WAL flush and wait for syncrep.
    // 	 * It's sufficient to ensure the toplevel transaction has an xid, no need
    // 	 * to assign xids subxacts, that'll already trigger an appropriate wait.
    // 	 * (Have to do that here, so we're outside the critical section)
    // 	 */
    if logit && relation_needs_wal(seqrel) {
        pg_sys::GetTopTransactionId();
    }

    // /* ready to change the on-disk (or really, in-buffer) tuple */
    // START_CRIT_SECTION();
    pg_sys::CritSectionCount += 1;

    // /*
    // 	 * We must mark the buffer dirty before doing XLogInsert(); see notes in
    // 	 * SyncOneBuffer().  However, we don't apply the desired changes just yet.
    // 	 * This looks like a violation of the buffer update protocol, but it is in
    // 	 * fact safe because we hold exclusive lock on the buffer.  Any other
    // 	 * process, including a checkpoint, that tries to examine the buffer
    // 	 * contents will block until we release the lock, and then will see the
    // 	 * final state that we install below.
    // 	 */
    pg_sys::MarkBufferDirty(buf);

    // /* XLOG stuff */
    if logit && relation_needs_wal(seqrel) {
        let mut xlrec: XlSeqRec = XlSeqRec::default();
        let recptr: pg_sys::XLogRecPtr;

        // /*
        //  * We don't log the current state of the tuple, but rather the state
        // 	* as it would appear after "log" more fetches.  This lets us skip
        // 	* that many future WAL records, at the cost that we lose those
        // 	* sequence values if we crash.
        // 	*/
        pg_sys::XLogBeginInsert();
        pg_sys::XLogRegisterBuffer(0, buf, pg_sys::REGBUF_WILL_INIT as u8);

        // /* set values that will be saved in xlog */
        (*seq).last_value = next;
        (*seq).is_called = true;
        (*seq).log_cnt = 0;

        xlrec.locator = (*seqrel).rd_locator;
        XLogRegisterData(
            (&mut xlrec as *mut XlSeqRec).cast::<std::ffi::c_char>(),
            size_of::<XlSeqRec>() as pg_sys::uint32
        );
        XLogRegisterData(
            seqdatatuple.t_data as *mut std::ffi::c_char,
            seqdatatuple.t_len
        );

        recptr = pg_sys::XLogInsert(pg_sys::RmgrIds::RM_SEQ_ID as pg_sys::RmgrId, XLOG_SEQ_LOG);
        pg_sys::PageSetLSN(page, recptr);
    }

    // /* Now update sequence tuple to the intended final state */
    (*seq).last_value = last; // /* last fetched number */
    (*seq).is_called = true;
    (*seq).log_cnt = log;

    // END_CRIT_SECTION();
    assert!(pg_sys::CritSectionCount > 0);
    pg_sys::CritSectionCount -= 1;

    let close_buffer_and_unlock_sequence = || {
        pg_sys::UnlockReleaseBuffer(buf);
        sequence_close(seqrel, pg_sys::NoLock);
    };

    // If grab_advisory_lock() throws an error(e.g. OutOfMemoryError) - we must ensure the buffer
    // and the sequence are closed. In this case, we can re-raise the error, which will cause the
    // potential transaction within which this code is executed to be rolled back.
    PgTryBuilder::new(|| {
        grab_advisory_lock(result);
    }).catch_others(|error| {
        close_buffer_and_unlock_sequence();
        error.rethrow();
    }).execute();

    close_buffer_and_unlock_sequence();

    result
}

unsafe fn init_sequence(relid: pg_sys::Oid, p_elm: *mut SeqTable, p_rel: *mut pg_sys::Relation) {
    let elm: SeqTable;
    let seqrel: pg_sys::Relation;
    let mut found: bool = false;

    // ensure hash table exists
    let htab = get_seq_hashtab();

    elm = pg_sys::hash_search(
        htab,
        (&relid as *const pg_sys::Oid).cast::<std::ffi::c_void>(),
        pg_sys::HASHACTION::HASH_ENTER,
        &mut found,
    ) as SeqTable;

    // /*
    //  * Initialize the new hash table entry if it did not exist already.
    //  *
    //  * NOTE: seqhashtab entries are stored for the life of a backend (unless
    //  * explicitly discarded with DISCARD). If the sequence itself is deleted
    //  * then the entry becomes wasted memory, but it's small enough that this
    //  * should not matter.
    //  */
    if !found {
        (*elm).filenumber = pg_sys::InvalidOid;
        (*elm).lxid = pg_sys::InvalidLocalTransactionId;
        (*elm).last_valid = false;
        (*elm).last = 0;
        (*elm).cached = 0;
    }
    seqrel = lock_and_open_sequence(elm);

    // /*
    //  * If the sequence has been transactionally replaced since we last saw it,
    //  * discard any cached-but-unissued values.  We do not touch the currval()
    //  * state, however.
    //  */
    if (*(*seqrel).rd_rel).relfilenode != (*elm).filenumber {
        (*elm).filenumber = (*(*seqrel).rd_rel).relfilenode;
        (*elm).cached = (*elm).last;
    }
    *p_elm = elm;
    *p_rel = seqrel;

}

unsafe fn read_seq_tuple(rel: pg_sys::Relation, buf: *mut pg_sys::Buffer,
                         seqdatatuple: pg_sys::HeapTuple) -> Form_pg_sequence_data {
    *buf = pg_sys::ReadBuffer(rel, 0);
    pg_sys::LockBuffer(*buf, pg_sys::BUFFER_LOCK_EXCLUSIVE as c_int);
    let page = pg_sys::BufferGetPage(*buf);
    let sm = pg_sys::PageGetSpecialPointer(page) as *mut SequenceMagic;
    if (*sm).magic != SEQ_MAGIC {
        pg_sys::ereport!(
            pg_sys::elog::PgLogLevel::ERROR,
            pg_sys::errcodes::PgSqlErrorCode::ERRCODE_DATA_CORRUPTED,
            format!(
                "bad magic number in sequence {:?} {:?}",
                (*(*rel).rd_rel).relname,
                (*sm).magic
            ),
        );
    }

    let lp = pg_sys::PageGetItemId(page, pg_sys::FirstOffsetNumber);
    // ItemIdIsNormal()
    assert_eq!((*lp).lp_flags(), pg_sys::LP_NORMAL);

    (*seqdatatuple).t_data = pg_sys::PageGetItem(page, lp) as pg_sys::HeapTupleHeader;
    (*seqdatatuple).t_len = (*lp).lp_len();

    // /*
    // 	 * Previous releases of Postgres neglected to prevent SELECT FOR UPDATE on
    // 	 * a sequence, which would leave a non-frozen XID in the sequence tuple's
    // 	 * xmax, which eventually leads to clog access failures or worse. If we
    // 	 * see this has happened, clean up after it.  We treat this like a hint
    // 	 * bit update, ie, don't bother to WAL-log it, since we can certainly do
    // 	 * this again if the update gets lost.
    // 	 */
    assert_eq!((*(*seqdatatuple).t_data).t_infomask & pg_sys::HEAP_XMAX_IS_MULTI as u16, 0);
    if (*(*seqdatatuple).t_data).t_choice.t_heap.t_xmax != pg_sys::InvalidTransactionId {
        (*(*seqdatatuple).t_data).t_choice.t_heap.t_xmax = pg_sys::InvalidTransactionId;
        (*(*seqdatatuple).t_data).t_infomask &= !pg_sys::HEAP_XMAX_COMMITTED as u16;
        (*(*seqdatatuple).t_data).t_infomask |= pg_sys::HEAP_XMAX_INVALID as u16;
        pg_sys::MarkBufferDirtyHint(*buf, true);
    }
    pg_sys::GETSTRUCT(seqdatatuple) as Form_pg_sequence_data
}

unsafe fn lock_and_open_sequence(seq: SeqTable) -> pg_sys::Relation {
    let thislxid = current_proc_lx_id();
    if (*seq).lxid != thislxid {
        let current_owner = pg_sys::CurrentResourceOwner;
        pg_sys::CurrentResourceOwner = pg_sys::TopTransactionResourceOwner;
        pg_sys::LockRelationOid((*seq).relid, pg_sys::RowExclusiveLock as pg_sys::LOCKMODE);
        pg_sys::CurrentResourceOwner = current_owner;
        (*seq).lxid = thislxid;
    }

    sequence_open(seq)
}

#[cfg(any(feature = "pg16"))]
unsafe fn current_proc_lx_id() -> pg_sys::LocalTransactionId {
    pg_sys::MyProc.as_ref().unwrap().lxid
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
unsafe fn current_proc_lx_id() -> pg_sys::LocalTransactionId {
    pg_sys::MyProc.as_ref().unwrap().vxid.lxid
}

unsafe fn sequence_open(seq: SeqTable) -> pg_sys::Relation {
    let relation = pg_sys::relation_open((*seq).relid, pg_sys::NoLock as pg_sys::LOCKMODE);
    // validate_relation_kind()
    if (*(*relation).rd_rel).relkind != pg_sys::RELKIND_SEQUENCE as std::ffi::c_char {
        pg_sys::ereport!(
            pg_sys::elog::PgLogLevel::ERROR,
            pg_sys::errcodes::PgSqlErrorCode::ERRCODE_WRONG_OBJECT_TYPE,
            format!("cannot open relation {:?}", (*(*relation).rd_rel).relname),
        );
    }
    relation
}

#[allow(non_snake_case)]
#[cfg(any(feature = "pg16", feature = "pg17"))]
unsafe fn XLogRegisterData(data: *mut std::ffi::c_char, len: u32) {
    pg_sys::XLogRegisterData(data, len);
}

#[allow(non_snake_case)]
#[cfg(any(feature = "pg18"))]
unsafe fn XLogRegisterData(data: *mut std::ffi::c_char, len: u32) {
    pg_sys::XLogRegisterData(data.cast::<std::ffi::c_void>(), len);
}

// #define XLogIsNeeded()
unsafe fn x_log_is_needed() -> bool {
    pg_sys::wal_level >= pg_sys::WalLevel::WAL_LEVEL_REPLICA as i32
}

// #define RelationIsPermanent(relation)
unsafe fn relation_is_permanent(relation: pg_sys::Relation) -> bool {
    (*(*relation).rd_rel).relpersistence as u8 == pg_sys::RELPERSISTENCE_PERMANENT
}

// #define RelationNeedsWAL(relation)
unsafe fn relation_needs_wal(relation: pg_sys::Relation) -> bool {
    relation_is_permanent(relation) &&
        (
            x_log_is_needed() || (
                (*relation).rd_createSubid == InvalidSubTransactionId &&
                    (*relation).rd_firstRelfilelocatorSubid == InvalidSubTransactionId)
        )
}

unsafe fn sequence_close(relation: pg_sys::Relation, lock_mode: u32) {
    pg_sys::relation_close(relation, lock_mode as pg_sys::LOCKMODE);
}

// --- end PostgreSQL-derived notice ---


#[macro_export]
macro_rules! grab_advisory_lock {
    ( $x:expr ) => {{
        let mut tag: pg_sys::LOCKTAG = std::mem::zeroed();

        let key = $x as u64;
        // Equivalent to SET_LOCKTAG_INT64(tag, key64) from src/backend/utils/adt/lockfuncs.c
        tag.locktag_field1 = pg_sys::MyDatabaseId.to_u32();
        tag.locktag_field2 = (key >> 32) as u32;
        tag.locktag_field3 = key as u32;
        tag.locktag_field4 = 1; // 1 means we are using i64 value which we split among field2 and field3
        tag.locktag_type = pg_sys::LockTagType::LOCKTAG_ADVISORY as u8;
        // USER_LOCKMETHOD for transaction level advisory locks
        tag.locktag_lockmethodid = pg_sys::USER_LOCKMETHOD as u8;

        pg_sys::LockAcquire(
            &mut tag as *mut pg_sys::LOCKTAG,
            pg_sys::ExclusiveLock as pg_sys::LOCKMODE,
            false,
            true,
        );
    }};
}

#[cfg(not(feature = "lock_tests"))]
unsafe fn grab_advisory_lock(id: i64) {
    grab_advisory_lock!(id);
}

#[cfg(feature = "lock_tests")]
unsafe fn grab_advisory_lock(id: i64) {
    let oid = pg_sys::GetSessionUserId();
    let name_ptr = pg_sys::GetUserNameFromId(oid, false);
    let uname = std::ffi::CStr::from_ptr(name_ptr).to_string_lossy().into_owned();
    if uname == "lock_tests_thread1" {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
    grab_advisory_lock!(id);
}

#[allow(dead_code)]
fn debug_log(str: &str) {
    pg_sys::log!("{}", str);
}

#[pg_extern]
unsafe fn nextval_with_xact_lock(oid: pg_sys::Oid) -> i64 {
    nextval_with_xact_lock_internal(oid, true)
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    #[pg_test]
    unsafe fn test_hello_nextval_with_xact_lock() {
        // assert_eq!(1, crate::nextval_with_xact_lock(pg_sys::Oid::from_u32(1u32)));
    }
}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    #[must_use]
    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
