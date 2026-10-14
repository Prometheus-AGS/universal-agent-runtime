BEGIN TRANSACTION; LET $value = 1; THROW 'uar_credential_mapping_conflict'; COMMIT TRANSACTION;
