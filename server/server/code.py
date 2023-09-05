import websockets
import asyncio
import json
import psycopg2

PORT = 8080
print("Server listening on Port " + str(PORT))

connected = set()


async def lapce_server(websocket, path):
    print("A client just connected")
    connected.add(websocket)

    hostname = 'localhost'
    username = 'beringtafa'
    password = 'your_password'
    database = 'beringtafa'
    portname = '5432'

    try:
        connection = psycopg2.connect(
            host=hostname,
            user=username,
            password=password,
            dbname=database,
            port=portname
        )
        print("Connected to the PostgreSQL database!")

        async for message in websocket:
            print("Received message from client: " + message)
            data = json.loads(message)
            if data.get("action") == "start_collab":
                file_name = data.get("file_name")
                file_content = data.get("file_content")
                try:
                    cursor = connection.cursor()
                    insert_query = "INSERT INTO collaborative_database (file_name, file_content) VALUES (%s, %s) RETURNING id"
                    cursor.execute(insert_query, (file_name, file_content))
                    loaded_file_content = cursor.fetchone()
                    connection.commit()
                    print("File data inserted into the database.")
                    cursor.close()
                    if loaded_file_content is not None:
                        response = {"status": "success", "id": loaded_file_content[0]}
                    else:
                        response = {"status": "success", "id": -1}
                    response_json = json.dumps(response)
                    await websocket.send(response_json)
                except psycopg2.Error as e:
                    print("Error inserting data into the database:", e)
            elif data.get("action") == "join_collab":
                session_id = data.get("session_id")
                try:
                    cursor = connection.cursor()
                    select_query = "SELECT file_content FROM collaborative_database WHERE id = %s"
                    cursor.execute(select_query, (session_id,))
                    loaded_file_content = cursor.fetchone()
                    cursor.close()
                    if loaded_file_content is not None:
                        loaded_file_content = loaded_file_content[0]
                        response = {"status": "success", "file_content": loaded_file_content}
                        response_json = json.dumps(response)
                        await websocket.send(response_json)
                    else:
                        response = {"status": "error", "file_content": "file_not_found_lapce_error_collab_gnireb_220223"}
                        response_json = json.dumps(response)
                        await websocket.send(response_json)
                except psycopg2.Error as e:
                    print("Error retrieving data from the database:", e)
            elif data.get("action") == "new_change":
                response_json = message
                for conn in connected:
                    try:
                        await conn.send(response_json)
                    except Exception as e:
                        print(f"Failed to send message. Error: {str(e)}")
            else:
                print("Invalid Message")
    except psycopg2.Error as e:
        print("Error connecting to the PostgreSQL database:", e)
    except websockets.exceptions.ConnectionClosed as e:
        print("A client just disconnected")
    finally:
        connected.remove(websocket)
        if connection is not None:
            connection.close()
            print("Connection closed.")

start_server = websockets.serve(lapce_server, "localhost", PORT)
asyncio.get_event_loop().run_until_complete(start_server)
asyncio.get_event_loop().run_forever()
