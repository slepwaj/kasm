import inputMethodEngine from '@ohos.inputMethodEngine';
import fs from '@ohos.file.fs';
import { common } from '@kit.AbilityKit';
import { create, key, backspace, space, select, setScheme, destroy } from 'libkasm.so';

const ROWS: string[] = ['qwertyuiop', 'asdfghjkl', 'zxcvbnm'];

interface Frame {
  preedit: string;
  candidates: string[];
  commit?: string;
}

@Entry
@Component
struct Keyboard {
  @State preedit: string = '';
  @State candidates: string[] = [];
  @State pageIndex: number = 0;
  @State schemeIndex: number = 0;
  private readonly PAGE_SIZE: number = 9;
  private readonly SCHEMES: string[] = ['全拼', '粤拼', '通字', 'tupa', '上古音'];
  private readonly SCHEME_KEYS: string[] = ['', 'jyutping', 'tungdzih', 'tupa', 'shanggu'];
  private engine: number = 0;

  aboutToAppear(): void {
    const context = getContext(this) as common.ExtensionContext;
    const dictPath = context.filesDir + '/dict.tsv';
    const writeRaw = (asset: string, target: string): void => {
      try {
        fs.accessSync(target);
      } catch (error) {
        const data = context.resourceManager.getRawFileContentSync(asset);
        const file = fs.openSync(target, fs.OpenMode.CREATE | fs.OpenMode.READ_WRITE);
        fs.writeSync(file.fd, data.buffer);
        fs.closeSync(file);
      }
    };
    writeRaw('dict.tsv', dictPath);
    const lexiconDir = context.filesDir + '/lexicon';
    try {
      fs.mkdirSync(lexiconDir);
    } catch (error) {
      // 已经存在
    }
    for (const name of this.SCHEME_KEYS.slice(1)) {
      writeRaw('lexicon/' + name + '.tsv', lexiconDir + '/' + name + '.tsv');
    }
    this.engine = create(dictPath);
  }

  aboutToDisappear(): void {
    if (this.engine !== 0) {
      destroy(this.engine);
      this.engine = 0;
    }
  }

  private apply(frame: Frame): void {
    this.preedit = frame.preedit;
    this.candidates = frame.candidates;
    this.pageIndex = 0;
  }

  private pageCount(): number {
    return Math.max(1, Math.ceil(this.candidates.length / this.PAGE_SIZE));
  }

  private pageCandidates(): string[] {
    const start = this.pageIndex * this.PAGE_SIZE;
    return this.candidates.slice(start, start + this.PAGE_SIZE);
  }

  private selectCandidate(index: number): void {
    if (this.engine === 0) {
      return;
    }
    const absolute = this.pageIndex * this.PAGE_SIZE + index;
    const frame = JSON.parse(select(this.engine, absolute)) as Frame;
    const client = AppStorage.get<inputMethodEngine.InputClient>('kasm.client');
    if (frame.commit !== undefined && client !== undefined) {
      client.insertText(frame.commit);
    }
    this.apply(frame);
  }

  private type(letter: string): void {
    if (this.engine === 0) {
      return;
    }
    this.apply(JSON.parse(key(this.engine, letter.charCodeAt(0))) as Frame);
  }

  private cycleScheme(): void {
    if (this.engine === 0) {
      return;
    }
    this.schemeIndex = (this.schemeIndex + 1) % this.SCHEMES.length;
    setScheme(this.engine, this.SCHEME_KEYS[this.schemeIndex]);
    this.preedit = '';
    this.candidates = [];
    this.pageIndex = 0;
  }

  private commitFirst(): void {
    if (this.engine === 0) {
      return;
    }
    const frame = JSON.parse(space(this.engine)) as Frame;
    const client = AppStorage.get<inputMethodEngine.InputClient>('kasm.client');
    if (frame.commit !== undefined && client !== undefined) {
      client.insertText(frame.commit);
    }
    this.apply(frame);
  }

  private removeLetter(): void {
    if (this.engine === 0) {
      return;
    }
    this.apply(JSON.parse(backspace(this.engine)) as Frame);
  }

  build() {
    Column({ space: 6 }) {
      Row({ space: 8 }) {
        Text('方案：' + this.SCHEMES[this.schemeIndex])
          .fontSize(13)
          .fontColor('#1B2A4A')
          .padding({ left: 6, right: 6, top: 2, bottom: 2 })
          .backgroundColor('#DCE3EE')
          .borderRadius(4)
          .onClick(() => this.cycleScheme())
      }
      .width('100%')
      .padding({ left: 12, right: 12 })

      Row({ space: 14 }) {
        Text(this.preedit.length > 0 ? this.preedit : 'KASM')
          .fontSize(18)
          .fontColor('#666666')
        ForEach(this.pageCandidates(), (candidate: string, index: number) => {
          Text(candidate)
            .fontSize(22)
            .fontColor('#111111')
            .onClick(() => this.selectCandidate(index))
        }, (candidate: string) => candidate)
        if (this.pageCount() > 1) {
          Text('‹')
            .fontSize(22)
            .fontColor(this.pageIndex > 0 ? '#111111' : '#BBBBBB')
            .onClick(() => {
              if (this.pageIndex > 0) {
                this.pageIndex -= 1;
              }
            })
          Text(`${this.pageIndex + 1}/${this.pageCount()}`)
            .fontSize(12)
            .fontColor('#888888')
          Text('›')
            .fontSize(22)
            .fontColor(this.pageIndex + 1 < this.pageCount() ? '#111111' : '#BBBBBB')
            .onClick(() => {
              if (this.pageIndex + 1 < this.pageCount()) {
                this.pageIndex += 1;
              }
            })
        }
      }
      .height(48)
      .width('100%')
      .padding({ left: 12, right: 12 })
      .backgroundColor('#F2F3F5')

      ForEach(ROWS, (row: string) => {
        Row({ space: 4 }) {
          ForEach(row.split(''), (letter: string) => {
            Button(letter.toUpperCase())
              .layoutWeight(1)
              .height(46)
              .fontSize(18)
              .backgroundColor('#FFFFFF')
              .fontColor('#111111')
              .onClick(() => this.type(letter))
          }, (letter: string) => letter)
        }
        .width('100%')
        .padding({ left: 4, right: 4 })
      }, (row: string) => row)

      Row({ space: 6 }) {
        Button('⌫')
          .layoutWeight(1)
          .height(46)
          .onClick(() => this.removeLetter())
        Button('空格')
          .layoutWeight(3)
          .height(46)
          .onClick(() => this.commitFirst())
      }
      .width('100%')
      .padding({ left: 4, right: 4 })
    }
    .width('100%')
    .backgroundColor('#E8EAED')
    .padding({ top: 6, bottom: 10 })
  }
}
