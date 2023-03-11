import whisper
import gc

def transcribe(filename):
    # TODO This creates memory that can't be properly freed, I believe
    # Leads to a `free(): invalid pointer` error on application close
    model = whisper.load_model("base.en")
    result = model.transcribe(filename)
    return result["text"]
