import express from 'express'

const app = express()


app.get("/", (req, res) => {
  console.log(req.url.toString())
  res.json({
    message: "Yo yo yo"
  })
})


const port = process.env.PORT || 8000

app.listen(port, () => {
  console.log(`Server listening on port ${port}`)
})
