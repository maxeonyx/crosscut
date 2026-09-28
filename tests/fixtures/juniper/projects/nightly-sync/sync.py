# Copies the day's ledger export to the backup bucket. Run from cron at 02:00.
import shutil
shutil.copy("/srv/ledger/export.csv", "/mnt/backup/ledger/")
