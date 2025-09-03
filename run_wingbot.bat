@echo off
REM Run the backend server in a new terminal window
start "Wingbot Server" cmd /k "cd wingbot_server && .\venv\Scripts\activate && uvicorn main:app --reload"
 
REM Run the Flutter client for Windows in the current window
cd wingbot_client
flutter run -d windows 