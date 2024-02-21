"""Setup script to initalize database table(s) with dummy data."""
import sqlite3


con = sqlite3.connect('pypeep.db')
cur = con.cursor()
cur.execute('CREATE TABLE requirements(id, name, current_version)')

params = list()
for i, dep in enumerate(('flask', 'pydantic', 'pandas')):
    params.append((i + 1, dep, None))
cur.executemany('INSERT INTO requirements VALUES(?, ?, ?)', params)
con.commit()
cur.close()
con.close()
