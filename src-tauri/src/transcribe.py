import whisper

def transcribe(filename):
    model = whisper.load_model("base.en")
    result = model.transcribe(filename)
    return result["text"]
