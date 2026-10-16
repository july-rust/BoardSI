"""Synthetic fixtures only; prove fail-closed behavior, not original PH-08."""
import hashlib
import json
import tempfile
import unittest
import warnings
import zipfile
from pathlib import Path
from ph08_evidence_audit import inspect


def digest(b): return hashlib.sha256(b).hexdigest()


def fixture(path, *, alter=None, sums_override=None, source_key="sources", include_extra=False):
    src = b'print("public synthetic")\n'
    manifest = json.dumps({source_key: [{"path": "src/check.py", "sha256": digest(src)}]}).encode()
    files = {"src/check.py": src, "evidence/manifest.json": manifest}
    checksum = ("".join(f"{digest(v)}  {k}\n" for k,v in files.items())).encode()
    if sums_override is not None:
        checksum = sums_override
    if alter:
        files.update(alter)
    if include_extra:
        files["extra.txt"] = b"unexpected, no digest"
    files["SHA256SUMS"] = checksum
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_DEFLATED) as z:
        for name, body in files.items(): z.writestr(name, body)
    return digest(path.read_bytes())


class ArchiveAudit(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.path = Path(self.temp.name)/"synthetic.zip"

    def check(self, **k):
        sha = fixture(self.path, **k)
        return inspect(self.path, expected_sha256=sha, expected_entries=2, expected_sources=1)

    def test_verified_positive(self):
        r = self.check()
        self.assertEqual(r["result"], "PASS")
        self.assertEqual((r["checksum_entries"],r["source_hash_verified"]),(2,1))

    def test_external_sha_missing(self):
        fixture(self.path)
        self.assertEqual(inspect(self.path)["result"], "NOT_VERIFIED")

    def test_external_sha_tamper(self):
        fixture(self.path)
        r = inspect(self.path, expected_sha256="a"*64)
        self.assertEqual(r["result"],"FAIL")
        self.assertTrue(any("archive SHA" in a for a in r["failures"]))

    def test_corrupted_inner_member(self):
        r=self.check(alter={"src/check.py":b"UNTRUSTED EDIT"})
        self.assertEqual(r["result"],"FAIL")
        self.assertTrue(any("digest mismatch" in s for s in r["failures"]))

    def test_bad_inner_digest(self):
        r=self.check(sums_override=f'{"0"*64}  src/check.py\n'.encode())
        self.assertEqual(r["result"],"FAIL")

    def test_missing_member(self):
        r=self.check(sums_override=f'{"0"*64}  missing.bin\n'.encode())
        self.assertEqual(r["result"],"FAIL")

    def test_bad_sum_format(self):
        r=self.check(sums_override=b"NOT A SHA256SUMS ENTRY\n")
        self.assertEqual(r["result"],"FAIL")

    def test_checksum_path_escape(self):
        r=self.check(sums_override=f'{"0"*64}  ../unsafe\n'.encode())
        self.assertEqual(r["result"],"FAIL")

    def test_zip_path_escape(self):
        r=self.check(alter={"../bad.txt":b"x"})
        self.assertEqual(r["result"],"FAIL")

    def test_duplicate_zip_member(self):
        fixture(self.path)
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", UserWarning)
            with zipfile.ZipFile(self.path,"a") as z:
                z.writestr("src/check.py",b"duplicate")
        r=inspect(self.path,expected_sha256=digest(self.path.read_bytes()))
        self.assertEqual(r["result"],"FAIL")

    def test_unlisted_file(self):
        self.assertEqual(self.check(include_extra=True)["result"],"NOT_VERIFIED")

    def test_source_schema_unknown(self):
        self.assertEqual(self.check(source_key="unexpected_schema")["result"],"NOT_VERIFIED")

    def test_source_inventory_hash_mismatch(self):
        source=b'print(42)'
        manifest=json.dumps({"sources":[{"path":"src/check.py","sha256":"0"*64}]}).encode()
        items={"src/check.py":source,"evidence/manifest.json":manifest}
        items["SHA256SUMS"]=''.join(f'{digest(v)}  {name}\n' for name,v in items.items()).encode()
        with zipfile.ZipFile(self.path,"w") as z:
            for n,b in items.items(): z.writestr(n,b)
        r=inspect(self.path,expected_sha256=digest(self.path.read_bytes()),expected_entries=2,expected_sources=1)
        self.assertEqual(r["result"],"FAIL")

    def test_archive_size_guard(self):
        sha=fixture(self.path)
        self.assertEqual(inspect(self.path, expected_sha256=sha, max_bytes=5)["result"],"FAIL")

    def test_wrong_expected_counts(self):
        sha=fixture(self.path)
        self.assertEqual(inspect(self.path,expected_sha256=sha,expected_entries=338,expected_sources=20)["result"],"FAIL")

    def test_invalid_zip(self):
        self.path.write_bytes(b"not a zip")
        self.assertEqual(inspect(self.path,expected_sha256=digest(self.path.read_bytes()))["result"],"FAIL")




# Additional explicit Chat-only stage 5C/5D manifest checks.  The archival
# integrity PASS does not imply any PH-08 physics or original HMM Gate-Q PASS.
class ChatOnlyBundle(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.path=Path(self.temp.name)/"chat.zip"

    def create(self, *, bad=False, extra=False):
        filename="boardsi-ph08-stage5d-gateq-diagnostic.py"
        original=b"synthetic test code\n"
        mapping={filename:digest(original)}
        body={filename: original if not bad else b"altered source"}
        if extra: body["unlisted.txt"]=b"not covered"
        body["boardsi-ph08-stage5d-manifest.json"]=json.dumps({"files_sha256":mapping}).encode()
        with zipfile.ZipFile(self.path,"w") as z:
            for n,content in body.items(): z.writestr(n,content)
        return digest(self.path.read_bytes())

    def test_chat_bundle_verified(self):
        sha=self.create()
        r=inspect(self.path, expected_sha256=sha)
        self.assertEqual(r["result"],"PASS")
        self.assertIn("Chat-only",r["format"])
        self.assertEqual(r["checksum_verified"],1)

    def test_chat_bundle_modified_code(self):
        sha=self.create(bad=True)
        r=inspect(self.path,expected_sha256=sha)
        self.assertEqual(r["result"],"FAIL")

    def test_chat_bundle_unlisted(self):
        sha=self.create(extra=True)
        self.assertEqual(inspect(self.path,expected_sha256=sha)["result"],"NOT_VERIFIED")

    def test_chat_bundle_no_known_external_hash(self):
        self.create()
        self.assertEqual(inspect(self.path)["result"],"NOT_VERIFIED")

class DirectoryGuard(unittest.TestCase):
    def test_malicious_directory(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)/"unsafe.zip"
            with zipfile.ZipFile(path,"w") as z:
                z.writestr("../traversal/", b"")
            self.assertEqual(inspect(path,expected_sha256=digest(path.read_bytes()))["result"],"FAIL")

class SymlinkGuard(unittest.TestCase):
    def test_symlink_member_rejected(self):
        import stat
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp)/"link.zip"
            with zipfile.ZipFile(path,"w") as z:
                info=zipfile.ZipInfo("leak")
                info.create_system=3
                info.external_attr=(stat.S_IFLNK | 0o777) << 16
                z.writestr(info,"/secret")
            r=inspect(path,expected_sha256=digest(path.read_bytes()))
            self.assertEqual(r["result"],"FAIL")
            self.assertTrue(any("symlink" in f for f in r["failures"]))



class DuplicateManifestKeys(unittest.TestCase):
    def test_chat_only_manifest_rejects_duplicate_digest_key(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "chat.zip"
            body = b"synthetic"
            sha = digest(body)
            name = "boardsi-ph08-stage5d-manifest.json"
            raw = ('{"files_sha256":{"sample.txt":"' + sha +
                   '","sample.txt":"' + sha + '"}}').encode()
            with zipfile.ZipFile(path, "w") as z:
                z.writestr("sample.txt", body)
                z.writestr(name, raw)
            r = inspect(path, expected_sha256=digest(path.read_bytes()))
            self.assertEqual(r["result"], "FAIL")
            self.assertTrue(any("manifest JSON" in v for v in r["failures"]))

    def test_original_manifest_rejects_duplicate_nested_source_field(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "original.zip"
            body = b"synthetic"
            source_digest = digest(body)
            raw = ('{"sources":[{"path":"src/check.py","path":"src/check.py",'
                   '"sha256":"' + source_digest + '"}]}').encode()
            contents = {"src/check.py": body, "evidence/manifest.json": raw}
            sums = ''.join(f'{digest(v)}  {k}\n' for k, v in contents.items()).encode()
            with zipfile.ZipFile(path, "w") as z:
                for name, b in contents.items():
                    z.writestr(name, b)
                z.writestr("SHA256SUMS", sums)
            r = inspect(path, expected_sha256=digest(path.read_bytes()),
                        expected_entries=2, expected_sources=1)
            self.assertEqual(r["result"], "FAIL")
            self.assertTrue(any("invalid JSON" in v for v in r["failures"]))


if __name__ == "__main__":
    unittest.main(verbosity=2)
