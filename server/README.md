THESIS : COLLABORATIVE EDITING INTO LAPCE CODE EDITOR
websocket tester

JSON FORMATS:
-------------

- START COLLAB SESSION
{
    "action" : "start_collab",
    "file_name" : "code.cpp",
    "file_content" : "#include <iostream>\nint main(){return 0;}"
}

- JOIN COLLAB SESSION
{
  "action" : "join_collab",
  "session_id" : 7
}


- MAKE NEW CHANGE TO THE FILE
{
  "action": "new_change",
  "value": "B",
  "position_id": [
    {
      "number": "1",
      "user": "1"
    },
    {
      "number": "1",
      "user": "2"
    }
  ]
}
