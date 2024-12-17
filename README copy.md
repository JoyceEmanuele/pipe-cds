# computed-data-service

O computed-data-service foi criado para que diariamente tenhamos o processamento diário das telemetrias dos dispositivos(no caso da Laager verificamos com a API deles), junto à outras configurações dos dispositivos que são obtidas no API-Server, para que seja calculado algumas informações com intuito de salvar em um banco de dados para facilitar o uso com PowerBI.

## Funcionamento
O serviço computed-data-service realiza uma operação diária de rotina que é acionada pontualmente à meia-noite, processando os dados referentes ao dia anterior. Durante esse processo, o primeiro passo consiste em realizar uma requisição ao API-Server para obter todos os clientes armazenados no banco de produção. Em seguida, cada thread fica processando um cliente por vez, com os clientes sendo distribuídos dinamicamente para as threads disponíveis.

Para cada cliente, é feita uma nova requisição ao API-Server, desta vez para obter todas as unidades desse cliente. A partir da lista de unidades obtida, é então feita uma nova requisição para cada unidade, com o objetivo de coletar todas as informações relativas aos dispositivos presentes naquela unidade, junto à suas configurações para realizar os cálculos necessários.

Após obtermos uma lista de dispositivos da unidade, processamos simultaneamente as funções de processamento de cada dispositivo, com uma observação para o DAC que mudamos para processar até 5 por vez, essas mudanças nos demais dispositivos e no DAC foram necessárias para gerar mais paralelismo e melhorar a eficiência em termos de tempo de processamento. Para cada dispositivo(exceção Laager, que consumimos uma API externa), analisamos suas telemetrias armazenadas no DynamoDB e realizamos os processamentos necessários.

Por fim, o processamento da disponibilidade de todos os dispositivos daquela unidade é o último cálculo a ser realizado antes de avançar para a próxima unidade.

## Processamento de dados de energia
Atualmente, temos uma lógica que preenche buracos de histórico de consumo dos medidores de energia. 


Primeiro, preenchemos o consumo de cada dispositivo por hora, pegando o array de telemetrias de cada hora, onde o consumo dessa hora será -> última hora da telemetria.en_at_tri - primeira hora da telemetria.en_at_tri e se eu tiver o array de telemetrias da hora seguinte, será primeira telemetria da hora seguinte.en_at_tri - primeira telemetria da hora atual.en_at_tri.

Logo após isso, verificamos dentro das horas de cada dia, quais são as que estão com consumo = 0 por não ter telemetrias, a identificando como uma hora sem consumo válido.
Se eu encontro uma hora em que o array de telemetrias daquela hora só tem um item, e o array de telemetrias da próxima hora é vazio, identifico essa hora como uma sem consumo válido.
Se o consumo de uma hora der negativo ou maior que 1000, a identifico como uma hora sem consumo válido.
Se eu encontro uma hora sem consumo válido e ainda não encontrei o primeiro consumo válido do dia, eu não alterarei os dados dessa hora, que serão verificados na lógica que explicarei mais abaixo.
Se eu encontro uma hora sem consumo válido e já passei por uma hora com consumo válido, então eu salvo essa posição da hora sem consumo em um array e continuo procurando.

Logo após, se eu encontro uma outra hora com consumo logo após alguma(s) hora(s) sem consumo válido, eu então pego o último en_at_tri da hora anterior com consumo que eu encontrei e pego o primeiro en_at_tri da hora atual com consumo que eu encontrei, faço a diferença e divido pela quantidade de horas que se passaram entre elas. Então salvo para cada hora que estava entre essas duas horas com consumo, o resultado da da diferença do en_at_tri dividindo pela quantidade de horas, que resultará no consumo médio para essas horas faltantes.

Se esse consumo que foi feito a média for menor que 0 ou maior que 1000, salvo o consumo como 0 e identifico que essa média não é um consumo válido (casos de erro na telemetria de medidores).

Logo após preencher os buracos que foram possíveis preencher durante o dia, verificamos qual foi a primeira hora com consumo válido daquele dia já processado e então pegamos qual foi o último histórico com consumo válido para aquele circuito elétrico, preenchendo o buraco entre dias. 

Primeiramente tentamos preencher novamente essas horas, buscando no dynamoDB(caso o dispositivo tenha ficado algum tempo offline, quando ele voltar a ter telemetrias significa que podemos ter dados pros dias anteriores com saved_data = true), fazendo o processamento novamente desse(s) dia(s) offline, verificando buraco entre horas e dias. Mas se não encontrarmos dados ou o último consumo salvo não for igual à hora anterior do primeiro consumo válido encontrado, pegamos no dynamo o array de telemetrias da última hora de consumo válido no banco e o array de telemetrias da primeira hora de consumo válido que processamos no dia, com isso pego o último en_at_tri do array do último consumo válido que encontrei no banco e pego o primeiro en_at_tri do primeiro consumo válido encontrado do dia processado, logo faço a diferença e divido pela quantidade de horas, em seguida salvo para cada hora o resultado dessa operação. Com isso, preenchemos o buraco de horas sem consumo.

Da mesma forma, se esse consumo que foi feito a média for menor que 0 ou maior que 1000, salvo o consumo como 0 e identifico que essa média não é um consumo válido (casos de erro na telemetria de medidores).

Obs: Descartamos telemetrias com esses valores de en_at_tri -> Null, -1, 2147483647, 1845494299, 65535

## Processamento de dados de água
Atualmente, temos uma lógica que preenche buracos de histórico de consumo dos medidores de água. 

Para dispositivos da Laager, temos em um primeiro momento a população de consumos de litro para cada hora do dia e suas validações. Foi reparado que podem ocorrer telemetrias com leitura negativa pelo dispositivo, retornando ou não ao normal na telemetria seguinte. 

Dessa forma, podemos encontrar alguns exemplos de telemetrias em um determinado horário, segue a validação para cada situação:

- [], teremos um consumo inválido por falta de telemetria, sendo um buraco a ser tratado.
- [-40 de leitura, +40 de leitura], teremos um consumo inválido pois a leitura foi incongruente e voltou ao normal, sendo um buraco a ser tratado.
- [-40 de leitura], teremos um consumo inválido pois tivemos um valor incongruente de leitura, sendo um buraco a ser tratado.
- [+40 de leitura, -30 leitura], teremos um consumo válido e seu valor é 10 litros.
- [-30 de leitura, +40 leitura], teremos um consumo válido e seu valor é 10 litros.
- [0], teremos um consumo válido, pois a leitura demonstrou que não houve acréscimo de consumo.
- [0, 0], teremos um consumo válido, pois as leituras demonstraram que não houveram acréscimos de consumo.
- [+40 de leitura], teremos um consumo válido, pois a leitura demonstrou que houve um consumo de 40 litros.
- [+1 de leitura, +1 de leitura], teremos um consumo válido, pois as leituras demonstraram que houveram um consumo de 2 litros.

Por seguinte, com a separação dos consumos e suas validações, nomearemos como "buracos" o conjunto de horários com telemetria(s) inválida(s) dentre telemetrias válidas em um dia. Nessa parte do tratamento validaremos essas horas inválidas pegando a primeira telemetria válida depois desse conjunto de telemetria(s) inválida(s) e calculando a média de consumo da seguinte forma: primeiro consumo válido / horas com telemetrias inválidas:
Exemplo: 
- 2:00 válido com consumo igual a 0
- 3:00 inválido
- 4:00 inválido 
- 5:00 inválido
- 6:00 válido com consumo igual a 40

Após o tratamento:
- 2:00 consumo igual a 0
- 3:00 consumo igual a 10
- 4:00 consumo igual a 10 
- 5:00 consumo igual a 10
- 6:00 consumo igual a 10

Por fim, popularemos os últimos horários de consumos inválidos do dia anterior (caso haja) e primeiros horários de consumos inválidos no dia atual. Pra isso, manteremos a mesma lógica anterior, primeiro consumo válido / número de horas de consumos inválidos até o último consumo válido.

Para dispositivos DMA, manteremos lógicas de tratamento parecidas com de energia. Com isso, primeiro, preenchemos o consumo de cada dispositivo por hora, pegando o array de telemetrias de cada hora, onde o consumo dessa hora será -> última hora da telemetria.pulses - primeira hora da telemetria.pulses e se eu tiver o array de telemetrias da hora seguinte, será primeira telemetria da hora seguinte.pulses - primeira telemetria da hora atual.pulses.

Logo após isso, verificamos dentro das horas de cada dia, quais são as que estão com consumo = 0 por não ter telemetrias, a identificando como uma hora sem consumo válido.
Se eu encontro uma hora em que o array de telemetrias daquela hora só tem um item, e o array de telemetrias da próxima hora é vazio, identifico essa hora como uma sem consumo válido.
Se o consumo de uma hora der negativo, a identifico como uma hora sem consumo válido.
Se eu encontro uma hora sem consumo válido e ainda não encontrei o primeiro consumo válido do dia, eu não alterarei os dados dessa hora, que serão verificados na lógica que explicarei mais abaixo.
Se eu encontro uma hora sem consumo válido e já passei por uma hora com consumo válido, então eu salvo essa posição da hora sem consumo em um array e continuo procurando.

Logo após, se eu encontro uma outra hora com consumo logo após alguma(s) hora(s) sem consumo válido, eu então pego o último pulses da hora anterior com consumo que eu encontrei e pego o primeiro pulses da hora atual com consumo que eu encontrei, faço a diferença e divido pela quantidade de horas que se passaram entre elas. Então salvo para cada hora que estava entre essas duas horas com consumo, o resultado da diferença do pulses dividindo pela quantidade de horas, que resultará no consumo médio para essas horas faltantes.

Logo após preencher os buracos que foram possíveis preencher durante o dia, popularemos os últimos horários de consumos inválidos do dia anterior (caso haja) e primeiros horários de consumos inválidos no dia atual. Para isso, manteremos a mesma lógica anterior, primeiro consumo válido / número de horas de consumos inválidos até o último consumo válido.

## Rota para popular dias anteriores
Com a introdução do novo serviço em produção, surgiu a necessidade de preencher os dados para os dias anteriores à data em que o serviço começou a operar. Para atender a essa demanda, foi desenvolvida uma rota dedicada para popular esses dias específicos. Esta rota permite especificar uma data inicial e uma data final para o período desejado.
```sh
curl -i -X POST -H "Content-Type: application/json" -d "{\"start_date\":\"2024-03-21\",\"end_date\":\"2024-03-31\"}" http://127.0.0.1:8088/script_days/all
```

## 🛠️ Ferramentas

- O projeto foi desenvolvido utilizando `Rust` como principal linguagem, sua documentação é bem interessante e pode ser encontrada no próprio [site da linguagem](https://prev.rust-lang.org/pt-BR/documentation.html). 

## 💻 Pré-requisitos

Antes de começar, é necessário verificar se todos os requisitos estão instalados em seu computador:

### Versão mais recente da linguagem Rust
O tutorial de instalação pode ser encontrado no próprio [site da Rust](https://www.rust-lang.org/tools/install). 

### O diesel_cli deve estar instalado na maquina

```sh
cargo install diesel_cli --no-default-features --features postgres  
```

No Windows, caso continue encontre problema ao rodar o comando acima, adicione essa variável de ambiente 
```sh
PQ_LIB_DIR="C:\Program Files\PostgreSQL\versão aqui\lib,
```

Agora reinicie o computador e tente novamente

### PostgreSQL

Também pode ser necessário instalar o postgreSQL, [site do postgreSQL](https://www.postgresql.org/download/)

### Timescale
O tutorial de instalação pode ser encontrado no prório [site do timescale](https://docs.timescale.com/self-hosted/latest/install/)

### Após criar o database, é necessário configurar o timescale
Rode essa linha no postgreSQL.

`CREATE EXTENSION IF NOT EXISTS timescaledb;`

### Clone o repositório
https://github.com/dielenergia/computed-data-service.git
`git clone https://github.com/dielenergia/computed-data-service.git`

### Configure as credenciais
Em um arquivo `configfile.json5` crie as credenciais seguindo o exemplo `configfile_example.json5`

### Execute as migrations

```sh
diesel migration run --database-url postgres://user:uniquepassword@localhost:3306/database
```
