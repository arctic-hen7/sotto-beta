# Sotto

> **NOTE:** A new version of Sotto with many more features is under active development, and will be released some time in 2024. This version will no longer be maintained, however supports the same high-fidelity dictation, and will remain open-source.

Sotto is a dead-simple recording and transcription app built on top of OpenAI's [Whisper](https://github.com/openai/whisper), an artifical intelligence capable of transcribing speech with near-human precision in dozens of languages. This app aims to make this technology accessible to everyone by creating a 'point-and-click' experience for transcription.

## Who is this for?

Right now, Sotto is in early-stage development, and a lot of work has yet to be done on the project before it's stable for real-world usage in mission-critical applications, but, right now, if you're looking to play around with transcription, Sotto is a great choice for anyone!

I originally built this app for my visually-impaired grandfather, who has great difficulty typing and using a computer, but who also wanted to write his memoirs. With the release of an AI that had the level of transcription ability that could enable this, all that was needed was an app to bring it all together into an interface simple enough for him to use easily.

So, while the original target audience of this app was the elderly, and those who are visually-impaired, it is a fantastic choice for anyone looking to speed up any typing-driven workflow. For example, I write a daily journal each night, and I can now transcribe that much more quickly from my speech with Sotto! Similarly, I have friends using this for medical dictations --- **Sotto runs completely locally, never sending your data online**. This means there are no confidentiality worries: what you record stays yours. Even better, Sotto is written to be incredibly simple: you record, end the recording, and then transcribe. When you next record something, your old recording is toast, meaning confidential information is overwritten rapidly.

## Usage

1. Download the latest version from [the releases page](https://github.com/arctic-hen7/sotto/releases).
2. Extract the downloaded Sotto archive.
3. Inside that directory, run `bash install.sh` in a terminal. You will be prompted for your password to update the database of desktop applications on your computer.

On Linux, a `.desktop` file will be created for you, and you're done! On macOS, you will need to manually create the app's representation through Automator.

If you're on Windows, you will just have a `.exe` file, with none of these scripts. At this time, we don't support proper packaging for Windows, but we soon will!

## Where did 'Sotto' come from?

Well, the underlying AI is called *Whisper*, and, in music, we often refer to a passage as needing to be played *sotto voce*, meaning with a subdued voice: quietly and softly. So, *Sotto* seemed like a nice choice!

## License

This project is licensed under the MIT license, which you can see [here](https://github.com/arctic-hen7/sotto/blob/main/LICENSE).
