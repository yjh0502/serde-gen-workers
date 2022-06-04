import React from 'react';
import Modal from 'react-modal';
import './App.css';

const customStyles = {
  overlay: {
    zIndex: 100,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
  },
  content: {
    top: '50%',
    left: '50%',
    right: 'auto',
    bottom: 'auto',
    marginRight: '-50%',
    transform: 'translate(-50%, -50%)',
  },
};

function getEditor(elem, lang) {
  if (elem.editor) {
    return elem.editor;
  }

  window.ace.config.set("basePath", "https://cdnjs.cloudflare.com/ajax/libs/ace/1.5.3/");

  const editor = window.ace.edit(elem);

  editor.setTheme("ace/theme/monokai");
  editor.session.setMode(`ace/mode/${lang}`);
  elem.editor = editor;

  return editor;
}

Modal.setAppElement('#root');

class App extends React.Component {
  constructor(props) {
    super(props);
    this.srcRef = React.createRef();
    this.genRef = React.createRef();

    this.state = {
      tests: true,

      helpOpen: false,
    };
  }

  componentDidMount() {
    this.editor_src = getEditor(this.srcRef.current, 'json');
    this.editor_src.setValue('// gen code will be here');

    this.editor_gen = getEditor(this.genRef.current, 'rust');
    this.editor_src.setValue('{"hello": "world"}');
  }

  componentWillUnmount() {
  }

  submit() {
    const { tests } = this.state;
    const editor = getEditor(this.srcRef.current, 'json');
    let str = editor.getValue();
    try {
      // test on browser
      JSON.parse(str);
    } catch(e) {
      console.log(e);
      this.editor_gen.setValue(`// failed to parse JSON: {e}`, -1);
      return;
    }

    fetch(`/schema?tests=${tests}`, {
      method: 'POST',
      body: editor.getValue(),
    }).then((resp) => resp.text())
    .then((text) => {
      this.editor_gen.setValue(text, -1);
    });
  }

  openModal() {
    this.setState({helpOpen: true});
  }

  afterOpenModal() {
  }

  closeModal() {
    this.setState({helpOpen: false});
  }

  render() {
    const { tests, helpOpen } = this.state;

    const host = 'https://rustgen.jyu.workers.dev';
    const example1 = `curl -XPOST -d '{"hello":"world"}' '${host}/schema'`;
    const example2 = `curl -XPOST -d @input.json '${host}/schema' -o meta.json`;
    const example3 = `curl -sf 'https://api.github.com/meta' \\\n | curl -XPOST -d @- '${host}/schema?tests'`;

    return (
      <div className="App">
      <div>
      <h1>JSON to Rust code generator</h1>
      </div>
      <button onClick={this.submit.bind(this)} className="button button-generate">generate</button>

      <div style={{display: 'inline-block'}}>
        <input type="checkbox"
          onChange={(ev) => this.setState({tests: ev.target.checked})}
          checked={tests}/>
        <label className="label-inline">generate tests</label>
      </div>

      <span>
      paste your JSON here to generate Rust schema,
      or <a href="#" onClick={this.openModal.bind(this)}>use curl</a>
      </span>
      <div ref={this.srcRef} className="editor editor-src"/>

      generated
      <div ref={this.genRef} className="editor editor-gen"/>

      <Modal
        isOpen={helpOpen}
        onAfterOpen={this.afterOpenModal.bind(this)}
        onRequestClose={this.closeModal.bind(this)}
        style={customStyles}
        contentLabel="Example Modal"
      >
        <h2><code>curl</code> instructions</h2>
        <span>use <code>curl</code> to generate schema</span>
        <pre>{example1}</pre>
        <span>generate schema from JSON file, save to disk</span>
        <pre>{example2}</pre>
        <span>generate on-the-fly from HTTP API, with testcases</span>
        <pre>{example3}</pre>

        <button onClick={this.closeModal.bind(this)}>close</button>
      </Modal>

      </div>
    );
  }
}

export default App;
