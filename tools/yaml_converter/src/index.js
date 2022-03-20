const yaml = require ( 'js-yaml' );
const fs = require ( 'fs' );
const commandLineArgs = require ( 'command-line-args' );

function run () {
	const optionDefinitions = [
		{
			name: 'inputfile',
			type: String,
		}
	]
	
	const options = commandLineArgs ( optionDefinitions )
	
	const inputfile = options.inputfile;
	const outputfile = `./temp/${options.inputfile}.json`;
	
	if(fs.existsSync(outputfile)){
		fs.unlinkSync(outputfile)
	}
	
	const obj = yaml.load ( fs.readFileSync ( inputfile, { encoding: 'utf-8' } ) );
	
	// this code if you want to save
	fs.writeFileSync ( outputfile, JSON.stringify ( obj, null, 2 ) );
}

run ();
