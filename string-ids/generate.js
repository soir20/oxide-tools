const Jenkins = require('hash-jenkins')
const fs = require('node:fs')

const usage = 'usage: node generate.js <in_file> <out_file> <range_start> <range_end>'

const in_file = process.argv[2]
if (!in_file) {
    console.error(usage)
    process.exit(1)
}

const out_file = process.argv[3]
if (!out_file) {
    console.error(usage)
    process.exit(1)
}

const range_start = parseInt(process.argv[4], 10) || 0
const range_end = parseInt(process.argv[5], 10) || 1_000_000

const hash_table = {}
for (let i = range_start; i < range_end; i++) {
    hash_table[Jenkins.lookup2(`Global.Text.${i}`)] = i
}

const data = fs.readFileSync(in_file, 'utf8')
const str_entries = data.split('\r\n')

const id_table = {}
const not_found = new Set()
for (const str_entry of str_entries) {
    const [hash, _, str] = str_entry.split('\t', 3)
    if (hash in hash_table) {
        const id = hash_table[hash]
        id_table[id] = str
    } else {
        not_found.add(parseInt(hash, 10).toString(16))
    }
}

let out_str = ''
for (const [id, str] of Object.entries(id_table)) {
    out_str += `${id}\t${str}\n`
}

fs.writeFileSync(out_file, out_str)  
console.log(`Could not look up hashes:\n${[...not_found].join(' ')}`)
